//! Serving one immutable projected snapshot to a browser on loopback.
//!
//! decision:6 settles what this may do: a foreground process may serve exactly
//! one immutable projected snapshot to loopback behind a per-launch capability,
//! and may not persist, watch, bind remotely, or outlive its invocation. This
//! module is that process's boundary and nothing more. It does not interpret a
//! recording — [`crate::inspection`] already did that, once, before the listener
//! existed.
//!
//! # The snapshot is taken once
//!
//! [`Snapshot::load`] replays the recording, projects it, and serializes the
//! projection to a string. After that the file is closed and never consulted
//! again. Nothing re-reads, watches, tails, or refreshes it, so a recording that
//! changes underneath a running viewer changes nothing the viewer shows. That is
//! what "immutable snapshot" means here, and it is a property of the code rather
//! than a promise about it.
//!
//! # The capability is the protection, not the loopback binding
//!
//! A recording contains prompts, commands, absolute paths, file contents, tool
//! output, and any credential that passed through any of them. Loopback binding
//! keeps that off the network; it does not keep it away from every other process
//! and every other page on the same machine. So every response that could carry
//! recording data requires an unguessable per-launch [`Capability`], read from
//! the operating system's random source and never reused.
//!
//! A request without it gets a fixed 404 carrying no session id, no record
//! count, no schema version, and no quoted record — the same response an
//! authorized request for a path that does not exist gets, so the two cannot be
//! told apart.
//!
//! # Nothing recording-derived becomes markup
//!
//! The bundled page contains no recording data. Recording data leaves this
//! process on exactly one route, as `application/json` with `nosniff`, so there
//! is no path on which a payload string could be parsed as markup. The server
//! also never maps a request path onto the filesystem: the two assets are
//! compiled into the binary, so path traversal has nothing to traverse.
//!
//! # Nothing is logged
//!
//! No request line, no path, no query, no header, no timing. A request carries
//! the capability in its query string and a response carries evidence, and the
//! cheapest way to be sure neither reaches a terminal or a file is to write
//! nothing at all.

use std::fmt;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::error::{Error, Result};
use crate::inspection::inspect;
use crate::replay::{Replay, replay_file};

/// The inert page, compiled into the binary. No CDN, font, or third-party asset.
const VIEWER_HTML: &str = include_str!("assets/viewer.html");
/// The page's stylesheet, likewise compiled in.
const VIEWER_CSS: &str = include_str!("assets/viewer.css");
/// The workbench script, likewise compiled in. Served from its own route rather
/// than inlined, so the content security policy can be `script-src 'self'`
/// rather than admitting inline script.
const VIEWER_JS: &str = include_str!("assets/viewer.js");
/// Replaced with the per-launch capability when the viewer binds.
const CAPABILITY_PLACEHOLDER: &str = "{{CAPABILITY}}";

/// Bytes of entropy behind a capability. 256 bits, hex-encoded to 64 characters.
const CAPABILITY_BYTES: usize = 32;

/// Largest request head this server will read. A browser fetching two documents
/// sends far less; anything larger is not a client this process needs to serve.
const MAX_REQUEST_BYTES: u64 = 8 * 1024;

/// How long one connection may take to send its request head, and how long a
/// response may take to write. Bounded so a socket that opens and says nothing
/// releases its slot rather than holding it.
const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// How many connections may be in flight at once. A browser rendering one page
/// with one stylesheet and one fetch uses a handful; the rest of the headroom is
/// for the speculative connections browsers open and abandon.
const MAX_CONNECTIONS: usize = 64;

/// An unguessable per-launch secret, required on every response that could carry
/// recording data.
///
/// Deliberately does not implement [`fmt::Display`], and its [`fmt::Debug`] is
/// redacted, so it cannot reach a log or an error message by accident. Reaching
/// it requires calling [`Capability::as_str`] and meaning it.
pub struct Capability(String);

impl Capability {
    /// Draw a fresh capability from the operating system's random source.
    ///
    /// There is no fallback. A weaker source — a clock, a pid, a hash of the
    /// path — would produce something that looks like a secret and is not, and
    /// failing to start is a better outcome than serving a recording behind one.
    pub fn generate() -> Result<Capability> {
        let mut bytes = [0u8; CAPABILITY_BYTES];
        random_bytes(&mut bytes)?;
        let mut hex = String::with_capacity(CAPABILITY_BYTES * 2);
        for byte in bytes {
            fmt::Write::write_fmt(&mut hex, format_args!("{byte:02x}"))
                .expect("writing to a String cannot fail");
        }
        Ok(Capability(hex))
    }

    /// The capability as it appears in a URL.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether a candidate matches, compared without an early return.
    ///
    /// The length is not secret — it is fixed by the constant above — so
    /// comparing lengths first leaks nothing. The bytes are compared in full
    /// either way.
    fn matches(&self, candidate: &str) -> bool {
        let expected = self.0.as_bytes();
        let candidate = candidate.as_bytes();
        if expected.len() != candidate.len() {
            return false;
        }
        let mut difference = 0u8;
        for (left, right) in expected.iter().zip(candidate) {
            difference |= left ^ right;
        }
        difference == 0
    }
}

impl fmt::Debug for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Capability(<redacted>)")
    }
}

#[cfg(unix)]
fn random_bytes(buffer: &mut [u8]) -> Result<()> {
    let mut source = std::fs::File::open("/dev/urandom")?;
    source.read_exact(buffer)?;
    Ok(())
}

#[cfg(not(unix))]
fn random_bytes(_buffer: &mut [u8]) -> Result<()> {
    Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "no operating system random source is wired up for this platform, and this build will \
         not invent one; `witnessglass view` is unavailable here",
    )))
}

/// One recording, replayed, projected, serialized, and then closed.
///
/// Holds no file handle, no path, and no way back to the recording. Everything
/// it can ever serve was decided before it existed.
#[derive(Debug)]
pub struct Snapshot {
    projection: String,
    schema_version: Option<u64>,
    records: usize,
    truncated: bool,
}

impl Snapshot {
    /// Replay, validate, and project one recording, then let go of the file.
    ///
    /// A corrupt recording fails here, before a listener is bound and before a
    /// browser is opened, so nothing renders partial content as trustworthy. A
    /// truncated recording succeeds: its valid prefix is evidence, and the
    /// truncation travels into the projection as scope.
    pub fn load(recording: &Path) -> Result<Snapshot> {
        let replay = replay_file(recording)?;
        Snapshot::from_replay(&replay)
    }

    /// Project an already replayed recording.
    pub fn from_replay(replay: &Replay) -> Result<Snapshot> {
        let inspection = inspect(replay);
        // Serialized once, here, at startup. The projection borrows the replay,
        // and rendering it now is what makes the snapshot independent of both.
        let projection = serde_json::to_string(&inspection).map_err(Error::Serialize)?;
        Ok(Snapshot {
            projection,
            schema_version: replay.schema_version,
            records: replay.records.len(),
            truncated: replay.tail.is_truncated(),
        })
    }

    /// The serialized projection. Internal and unstable, not an interchange
    /// format.
    pub fn projection_json(&self) -> &str {
        &self.projection
    }

    /// How many complete records the projection covers.
    pub fn records(&self) -> usize {
        self.records
    }

    /// Schema version the recording declared, or `None` for a recording holding
    /// no complete records.
    pub fn schema_version(&self) -> Option<u64> {
        self.schema_version
    }

    /// Whether the recording this snapshot came from stops mid-record.
    pub fn is_truncated(&self) -> bool {
        self.truncated
    }
}

/// One additional document a caller may attach to a viewer.
///
/// Exists for exactly one reason: sprint:7 renders a second perspective over the
/// same immutable snapshot, and building it a second HTTP server would mean a
/// second implementation of the capability check, the headers, the
/// not-found equivalence, and the no-logging rule — with `tests/view.rs`
/// covering only one of them. So the transport carries an extra document instead,
/// and learns nothing about what is in it.
///
/// The route is `'static` because attachments come from compiled-in call sites,
/// never from a request, a path, or a configuration value. Nothing here maps a
/// request onto the filesystem, and adding an attachment does not change that.
/// An attachment cannot shadow a built-in route: [`Viewer::bind_with`] refuses
/// one that tries.
#[derive(Debug, Clone)]
pub struct Attachment {
    /// Absolute request path, beginning with `/`.
    pub route: &'static str,
    /// Content type served with it.
    pub content_type: &'static str,
    /// The body. `{{CAPABILITY}}` in it is substituted at bind time, exactly as
    /// it is in the built-in assets.
    pub body: String,
}

/// Routes the viewer answers on its own. An attachment may not take one.
const BUILT_IN_ROUTES: [&str; 4] = ["/", "/viewer.css", "/viewer.js", "/projection.json"];

/// A bound loopback listener serving one snapshot behind one capability.
///
/// Binding does not start serving. [`Viewer::serve_forever`] does, and it
/// returns only on an error that makes accepting impossible — the process is
/// expected to be ended by whoever started it.
#[derive(Debug)]
pub struct Viewer {
    listener: TcpListener,
    capability: Capability,
    snapshot: Snapshot,
    index: String,
    stylesheet: String,
    script: String,
    attachments: Vec<Attachment>,
    in_flight: AtomicUsize,
}

impl Viewer {
    /// Bind a listener on an operating-system-selected loopback port.
    ///
    /// The address is `127.0.0.1` and there is no parameter, environment
    /// variable, or flag anywhere in this crate that changes it. Remote binding
    /// is not a configuration this build can be talked into.
    pub fn bind(snapshot: Snapshot) -> Result<Viewer> {
        Viewer::bind_with(snapshot, Vec::new())
    }

    /// Bind, and answer some additional compiled-in documents.
    ///
    /// Every property [`Viewer::bind`] has holds here unchanged: loopback only,
    /// one per-launch capability, authorization decided before routing, the same
    /// headers on every response, nothing logged, and no request path ever
    /// reaching the filesystem. An attachment is answered *after* the capability
    /// check like everything else, so attaching a document does not create a way
    /// to read one without it.
    ///
    /// Refuses an attachment whose route collides with a built-in one or does not
    /// begin with `/`, so a caller cannot replace the projection or the page by
    /// accident.
    pub fn bind_with(snapshot: Snapshot, attachments: Vec<Attachment>) -> Result<Viewer> {
        for attachment in &attachments {
            if !attachment.route.starts_with('/') || BUILT_IN_ROUTES.contains(&attachment.route) {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "attachment route {:?} is not an absolute path outside the built-in \
                         routes",
                        attachment.route
                    ),
                )));
            }
        }

        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let capability = Capability::generate()?;

        // The capability is hex from a fixed alphabet, so substituting it into
        // markup cannot introduce a character that changes the document's
        // structure. Checked rather than assumed, because the check is free and
        // the assumption is the kind that stops being true quietly.
        debug_assert!(
            capability
                .as_str()
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit()),
            "a capability must be hex before it is substituted into markup"
        );

        let index = VIEWER_HTML.replace(CAPABILITY_PLACEHOLDER, capability.as_str());
        let stylesheet = VIEWER_CSS.replace(CAPABILITY_PLACEHOLDER, capability.as_str());
        let script = VIEWER_JS.replace(CAPABILITY_PLACEHOLDER, capability.as_str());
        let attachments = attachments
            .into_iter()
            .map(|attachment| Attachment {
                body: attachment
                    .body
                    .replace(CAPABILITY_PLACEHOLDER, capability.as_str()),
                ..attachment
            })
            .collect();

        Ok(Viewer {
            listener,
            capability,
            snapshot,
            index,
            stylesheet,
            script,
            attachments,
            in_flight: AtomicUsize::new(0),
        })
    }

    /// The loopback address the listener actually got.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    /// The URL a browser needs, capability included.
    ///
    /// This string is a secret. It is printed once, to the terminal that asked
    /// for it, and it is never logged.
    pub fn url(&self) -> Result<String> {
        let addr = self.local_addr()?;
        Ok(format!(
            "http://{}:{}/?c={}",
            Ipv4Addr::LOCALHOST,
            addr.port(),
            self.capability.as_str()
        ))
    }

    /// The per-launch capability.
    pub fn capability(&self) -> &Capability {
        &self.capability
    }

    /// The snapshot being served.
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    /// Accept and answer connections until accepting fails.
    ///
    /// A connection that misbehaves is answered or dropped; it never ends the
    /// loop. Only the listener itself failing does.
    pub fn serve_forever(&self) -> Result<()> {
        // One thread per connection, bounded. Browsers open connections
        // speculatively and leave some of them idle, so answering strictly one
        // at a time would let a socket that says nothing hold the page hostage
        // for a whole read timeout. The bound exists because "loopback only" is
        // not the same as "nothing local can misbehave", and an unbounded spawn
        // on an accept loop is a thread bomb with a polite name.
        std::thread::scope(|scope| {
            loop {
                let (stream, _peer) = self.listener.accept()?;

                if self.in_flight.fetch_add(1, Ordering::AcqRel) >= MAX_CONNECTIONS {
                    // Over the bound: close without answering. Nothing is said
                    // about why, because saying anything is disclosure.
                    self.in_flight.fetch_sub(1, Ordering::AcqRel);
                    drop(stream);
                    continue;
                }

                scope.spawn(move || {
                    // A client that stalls is dropped rather than served. Its
                    // socket closing is the whole of the consequence.
                    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
                    let _ = stream.set_write_timeout(Some(READ_TIMEOUT));
                    let _ = self.answer(stream);
                    self.in_flight.fetch_sub(1, Ordering::AcqRel);
                });
            }
        })
    }

    fn answer(&self, mut stream: TcpStream) -> std::io::Result<()> {
        let Some(request) = read_request(&stream)? else {
            return respond(&mut stream, Status::BadRequest, None, b"", false);
        };

        // Only reading verbs exist. Anything else is refused without being told
        // whether the path it named is real.
        let head_only = match request.method.as_str() {
            "GET" => false,
            "HEAD" => true,
            _ => return respond(&mut stream, Status::NotAllowed, None, b"", false),
        };

        // Authorization is decided before routing, and an unauthorized request
        // gets the same response as an authorized request for a path that does
        // not exist. A caller cannot learn what exists here by asking.
        let authorized = request
            .capability
            .as_deref()
            .is_some_and(|candidate| self.capability.matches(candidate));
        if !authorized {
            return respond(
                &mut stream,
                Status::NotFound,
                Some(TEXT),
                NOT_FOUND,
                head_only,
            );
        }

        match request.path.as_str() {
            "/" => respond(
                &mut stream,
                Status::Ok,
                Some(HTML),
                self.index.as_bytes(),
                head_only,
            ),
            "/viewer.css" => respond(
                &mut stream,
                Status::Ok,
                Some(CSS),
                self.stylesheet.as_bytes(),
                head_only,
            ),
            "/viewer.js" => respond(
                &mut stream,
                Status::Ok,
                Some(JAVASCRIPT),
                self.script.as_bytes(),
                head_only,
            ),
            "/projection.json" => respond(
                &mut stream,
                Status::Ok,
                Some(JSON),
                self.snapshot.projection.as_bytes(),
                head_only,
            ),
            path => match self
                .attachments
                .iter()
                .find(|attachment| attachment.route == path)
            {
                Some(attachment) => respond(
                    &mut stream,
                    Status::Ok,
                    Some(attachment.content_type),
                    attachment.body.as_bytes(),
                    head_only,
                ),
                None => respond(
                    &mut stream,
                    Status::NotFound,
                    Some(TEXT),
                    NOT_FOUND,
                    head_only,
                ),
            },
        }
    }

    /// The routes this viewer answers, built-in and attached, for a caller that
    /// wants to print them.
    pub fn attachment_routes(&self) -> Vec<&'static str> {
        self.attachments
            .iter()
            .map(|attachment| attachment.route)
            .collect()
    }
}

/// Ask the desktop to open a URL.
///
/// Separate from [`Viewer`] on purpose: binding and serving never launch
/// anything, so `--no-open` is the absence of a call rather than a flag threaded
/// through the server. The URL carries the capability, so it is briefly visible
/// in this machine's process table — the same machine the recording is already
/// on, and the reason the capability is per-launch.
pub fn open_in_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    const OPENER: &str = "open";
    #[cfg(target_os = "linux")]
    const OPENER: &str = "xdg-open";

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        std::process::Command::new(OPENER)
            .arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = url;
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "no browser opener is wired up for this platform",
        ))
    }
}

/// The parts of a request this server acts on. Everything else is read and
/// discarded.
struct Request {
    method: String,
    path: String,
    capability: Option<String>,
}

/// Read a request head, bounded in both size and time.
///
/// Returns `None` for a head this server will not act on. No part of it is
/// retained beyond [`Request`], and none of it is logged.
fn read_request(stream: &TcpStream) -> std::io::Result<Option<Request>> {
    let mut reader = BufReader::new(stream).take(MAX_REQUEST_BYTES);
    let mut first = String::new();
    if reader.read_line(&mut first)? == 0 {
        return Ok(None);
    }

    // Drain the remaining head so the client is not answered mid-write. A
    // request body is never read, because no route accepts one.
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) if line.trim_end_matches(['\r', '\n']).is_empty() => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }

    let mut parts = first.trim_end_matches(['\r', '\n']).split(' ');
    let (Some(method), Some(target)) = (parts.next(), parts.next()) else {
        return Ok(None);
    };
    if method.is_empty() || !target.starts_with('/') {
        return Ok(None);
    }

    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (target, None),
    };

    // The capability is hex, so it needs no percent-decoding. Not writing a
    // decoder is one fewer thing to get wrong on the path that decides access.
    let capability = query.and_then(|query| {
        query
            .split('&')
            .find_map(|pair| pair.strip_prefix("c="))
            .map(str::to_owned)
    });

    Ok(Some(Request {
        method: method.to_owned(),
        path: path.to_owned(),
        capability,
    }))
}

/// The statuses this server can produce. There are four.
#[derive(Clone, Copy)]
enum Status {
    Ok,
    BadRequest,
    NotFound,
    NotAllowed,
}

impl Status {
    fn line(self) -> &'static str {
        match self {
            Status::Ok => "200 OK",
            Status::BadRequest => "400 Bad Request",
            Status::NotFound => "404 Not Found",
            Status::NotAllowed => "405 Method Not Allowed",
        }
    }
}

const HTML: &str = "text/html; charset=utf-8";
const CSS: &str = "text/css; charset=utf-8";
const JAVASCRIPT: &str = "text/javascript; charset=utf-8";
const JSON: &str = "application/json; charset=utf-8";
const TEXT: &str = "text/plain; charset=utf-8";

/// The only body an unauthorized or unrouted request ever gets. It names no
/// session, no record, no count, and no version, and it is byte-identical for
/// both cases.
const NOT_FOUND: &[u8] = b"not found\n";

/// Headers sent on every response, whatever the status.
///
/// `script-src 'self'` and no more: the workbench script is served from this
/// origin, and there is no inline script, no `eval`, and no nonce, so nothing
/// weaker is needed. `default-src 'none'` means every fetch destination that is
/// not named below is refused outright, which includes any attempt to reach off
/// this machine.
///
/// There is no `Server` header and no `Date` header. Neither is needed by a
/// browser on the same machine, and both are disclosure for nothing.
const SECURITY_HEADERS: &[&str] = &[
    "Cache-Control: no-store, no-cache, must-revalidate, max-age=0",
    "Pragma: no-cache",
    "X-Content-Type-Options: nosniff",
    "X-Frame-Options: DENY",
    "Referrer-Policy: no-referrer",
    "Content-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self'; \
     connect-src 'self'; img-src 'none'; font-src 'none'; object-src 'none'; base-uri 'none'; \
     form-action 'none'; frame-ancestors 'none'",
    "Cross-Origin-Opener-Policy: same-origin",
    "Cross-Origin-Resource-Policy: same-origin",
    "Cross-Origin-Embedder-Policy: require-corp",
    "Connection: close",
];

fn respond(
    stream: &mut TcpStream,
    status: Status,
    content_type: Option<&str>,
    body: &[u8],
    head_only: bool,
) -> std::io::Result<()> {
    let mut head = String::with_capacity(512);
    head.push_str("HTTP/1.1 ");
    head.push_str(status.line());
    head.push_str("\r\n");
    for header in SECURITY_HEADERS {
        head.push_str(header);
        head.push_str("\r\n");
    }
    if let Some(content_type) = content_type {
        head.push_str("Content-Type: ");
        head.push_str(content_type);
        head.push_str("\r\n");
    }
    // Present even on a HEAD, where it describes the body a GET would return.
    head.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));

    stream.write_all(head.as_bytes())?;
    if !head_only {
        stream.write_all(body)?;
    }
    stream.flush()
}
