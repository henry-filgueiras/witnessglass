//! The loopback snapshot server.
//!
//! Every recording here is synthetic. Nothing in this file reads, lists, copies,
//! or is derived from a real recording, and nothing makes an outbound request.
//!
//! The tests talk raw HTTP over a TcpStream rather than through a client
//! library, because the exact bytes on the wire — which headers, which status,
//! which body — are the thing under test.

mod common;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;

use common::*;
use witnessglass::view::{Capability, Snapshot, Viewer};
use witnessglass::{Context, Event, replay_bytes};

/// A row with no causal context supplied.
fn row(recorded_at: &str, event: Event) -> V2Row<'_> {
    (recorded_at, Context::default(), event)
}

/// A small, complete, unremarkable synthetic recording.
fn ordinary_recording() -> String {
    v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
        row("2026-01-01T00:00:03Z", ev_session_ended(Some("exit"))),
    ])
}

fn snapshot_of(recording: &str) -> Snapshot {
    let replay = replay_bytes(recording.as_bytes()).expect("synthetic recording should replay");
    Snapshot::from_replay(&replay).expect("a valid replay should project")
}

/// A bound viewer serving in a background thread for the rest of the test
/// process. The thread is deliberately detached: the process ending is what
/// stops the server, which is the lifetime the command has too.
struct Served {
    addr: SocketAddr,
    capability: String,
}

fn serve(snapshot: Snapshot) -> Served {
    let viewer = Viewer::bind(snapshot).expect("binding loopback should succeed");
    let addr = viewer
        .local_addr()
        .expect("a bound listener has an address");
    let capability = viewer.capability().as_str().to_owned();
    std::thread::spawn(move || {
        // Errors here end this connection, not the test.
        let _ = viewer.serve_forever();
    });
    Served { addr, capability }
}

/// One raw HTTP exchange. Returns the response head and body separately.
fn request(addr: SocketAddr, raw: &str) -> (String, String) {
    let mut stream = TcpStream::connect(addr).expect("connecting to the viewer should succeed");
    stream
        .write_all(raw.as_bytes())
        .expect("writing a request should succeed");
    stream.flush().expect("flushing should succeed");

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("reading a response should succeed");
    let response = String::from_utf8_lossy(&response).into_owned();
    match response.split_once("\r\n\r\n") {
        Some((head, body)) => (head.to_owned(), body.to_owned()),
        None => (response, String::new()),
    }
}

fn get(addr: SocketAddr, target: &str) -> (String, String) {
    request(
        addr,
        &format!("GET {target} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"),
    )
}

// ---------------------------------------------------------------------------
// Authorization
// ---------------------------------------------------------------------------

#[test]
fn an_unauthenticated_request_discloses_nothing_about_the_recording() {
    let served = serve(snapshot_of(&ordinary_recording()));

    for target in [
        "/",
        "/projection.json",
        "/viewer.css",
        "/?c=",
        "/projection.json?c=wrong",
        "/projection.json?c=0000000000000000000000000000000000000000000000000000000000000000",
    ] {
        let (head, body) = get(served.addr, target);
        assert!(
            head.starts_with("HTTP/1.1 404 Not Found"),
            "{target} should not be served without the capability, got: {head}"
        );
        assert_eq!(body, "not found\n", "for {target}");

        // Nothing about the recording, in the head or the body.
        let whole = format!("{head}{body}");
        for leak in [
            SESSION,
            "schema_version",
            "tool_use_id",
            "toolu_a",
            "SyntheticTool",
            "sequence",
            "records",
            "session_started",
        ] {
            assert!(
                !whole.contains(leak),
                "an unauthenticated response leaked {leak:?} for {target}"
            );
        }
    }
}

#[test]
fn an_unauthorized_request_and_an_unknown_path_are_indistinguishable() {
    let served = serve(snapshot_of(&ordinary_recording()));

    let (unauthorized_head, unauthorized_body) = get(served.addr, "/projection.json");
    let (unknown_head, unknown_body) = get(
        served.addr,
        &format!("/does-not-exist?c={}", served.capability),
    );

    assert_eq!(unauthorized_head, unknown_head);
    assert_eq!(unauthorized_body, unknown_body);
}

#[test]
fn the_capability_unlocks_the_page_the_stylesheet_and_the_projection() {
    let served = serve(snapshot_of(&ordinary_recording()));
    let c = &served.capability;

    let (head, body) = get(served.addr, &format!("/?c={c}"));
    assert!(head.starts_with("HTTP/1.1 200 OK"));
    assert!(head.contains("Content-Type: text/html; charset=utf-8"));
    assert!(body.contains("WitnessGlass"));
    // The three perspectives are declared in the served markup, not conjured by
    // script, so the page announces its own structure before anything runs.
    assert!(body.contains(r#"role="tablist""#));
    for perspective in ["Events", "Coverage", "Provenance"] {
        assert!(
            body.contains(perspective),
            "the page should offer {perspective:?}"
        );
    }

    let (head, body) = get(served.addr, &format!("/viewer.css?c={c}"));
    assert!(head.starts_with("HTTP/1.1 200 OK"));
    assert!(head.contains("Content-Type: text/css; charset=utf-8"));
    assert!(body.contains("--paper"));

    let (head, body) = get(served.addr, &format!("/viewer.js?c={c}"));
    assert!(head.starts_with("HTTP/1.1 200 OK"));
    assert!(head.contains("Content-Type: text/javascript; charset=utf-8"));
    assert!(body.contains("projection.json"));

    let (head, body) = get(served.addr, &format!("/projection.json?c={c}"));
    assert!(head.starts_with("HTTP/1.1 200 OK"));
    assert!(head.contains("Content-Type: application/json; charset=utf-8"));
    let parsed: serde_json::Value =
        serde_json::from_str(&body).expect("the projection endpoint should serve JSON");
    assert_eq!(parsed["schema_version"], 2);
    assert_eq!(parsed["records"].as_array().expect("records").len(), 4);
}

#[test]
fn a_capability_is_unguessable_fresh_per_launch_and_never_printed_by_debug() {
    let first = Capability::generate().expect("a random source should be available");
    let second = Capability::generate().expect("a random source should be available");

    assert_eq!(first.as_str().len(), 64);
    assert!(first.as_str().bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_ne!(first.as_str(), second.as_str());

    // It cannot reach a log by being formatted.
    let rendered = format!("{first:?}");
    assert_eq!(rendered, "Capability(<redacted>)");
    assert!(!rendered.contains(first.as_str()));

    // And two viewers over the same snapshot do not share one.
    let a = Viewer::bind(snapshot_of(&ordinary_recording())).expect("bind");
    let b = Viewer::bind(snapshot_of(&ordinary_recording())).expect("bind");
    assert_ne!(a.capability().as_str(), b.capability().as_str());
}

#[test]
fn only_reading_methods_are_served() {
    let served = serve(snapshot_of(&ordinary_recording()));
    let c = &served.capability;

    let (head, body) = request(
        served.addr,
        &format!(
            "POST /projection.json?c={c} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: \
             0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        head.starts_with("HTTP/1.1 405 Method Not Allowed"),
        "{head}"
    );
    assert_eq!(body, "");

    // HEAD is answered with the headers a GET would produce and no body.
    let (head, body) = request(
        served.addr,
        &format!(
            "HEAD /projection.json?c={c} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(head.starts_with("HTTP/1.1 200 OK"));
    assert!(head.contains("Content-Length: "));
    assert_eq!(body, "");
}

// ---------------------------------------------------------------------------
// Binding
// ---------------------------------------------------------------------------

#[test]
fn the_listener_binds_only_to_loopback_on_an_os_selected_port() {
    let viewer = Viewer::bind(snapshot_of(&ordinary_recording())).expect("bind");
    let addr = viewer.local_addr().expect("address");

    assert!(
        addr.ip().is_loopback(),
        "bound to {addr}, which is not loopback"
    );
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_ne!(addr.port(), 0, "the OS should have selected a real port");

    let url = viewer.url().expect("url");
    assert!(url.starts_with("http://127.0.0.1:"));
    assert!(url.contains(&format!(":{}/?c=", addr.port())));
    assert!(url.ends_with(viewer.capability().as_str()));
}

// ---------------------------------------------------------------------------
// Response headers
// ---------------------------------------------------------------------------

#[test]
fn every_response_carries_the_restrictive_headers() {
    let served = serve(snapshot_of(&ordinary_recording()));
    let c = &served.capability;

    for target in [
        format!("/?c={c}"),
        format!("/viewer.css?c={c}"),
        format!("/projection.json?c={c}"),
        "/projection.json".to_owned(),
    ] {
        let (head, _) = get(served.addr, &target);
        for header in [
            "Cache-Control: no-store, no-cache, must-revalidate, max-age=0",
            "Pragma: no-cache",
            "X-Content-Type-Options: nosniff",
            "X-Frame-Options: DENY",
            "Referrer-Policy: no-referrer",
            "Cross-Origin-Opener-Policy: same-origin",
            "Cross-Origin-Resource-Policy: same-origin",
            "Cross-Origin-Embedder-Policy: require-corp",
            "Connection: close",
        ] {
            assert!(head.contains(header), "{target} is missing {header:?}");
        }

        assert!(head.contains("Content-Security-Policy: default-src 'none'"));
        // `'self'` and no more: the workbench script is served from this origin
        // and there is no inline script and no `eval`, so nothing weaker is
        // needed.
        assert!(head.contains("script-src 'self'"));
        assert!(head.contains("style-src 'self'"));
        assert!(head.contains("frame-ancestors 'none'"));
        assert!(
            !head.contains("unsafe-inline"),
            "{target} admits inline script or style"
        );
        assert!(!head.contains("unsafe-eval"));

        // No disclosure for nothing.
        assert!(
            !head.contains("Server:"),
            "{target} discloses a Server header"
        );
        assert!(!head.contains("Set-Cookie"), "{target} sets a cookie");
    }
}

// ---------------------------------------------------------------------------
// Recording-derived content
// ---------------------------------------------------------------------------

#[test]
fn hostile_payload_strings_survive_as_text_and_never_as_markup() {
    let hostile = "</script><img src=x onerror=alert(1)><svg onload=alert(2)>\"'`";
    let recording = v2_recording(vec![
        row(
            "2026-01-01T00:00:00Z",
            ev_reported_intent(hostile, Some("toolu_a")),
        ),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_requested("toolu_a", hostile),
        ),
    ]);
    let served = serve(snapshot_of(&recording));
    let c = &served.capability;

    // The served page carries no recording data at all, hostile or otherwise.
    // That is the guarantee at this layer: no payload reaches a markup route,
    // because the page is a fixed document and the projection is JSON. What the
    // browser then does with it is guarded in `tests/workbench.rs` and verified
    // by hand against `docs/viewer.md`.
    let (_, page) = get(served.addr, &format!("/?c={c}"));
    assert!(!page.contains("onerror"));
    assert!(!page.contains("toolu_a"));
    assert!(!page.contains(hostile));

    // The projection carries it, intact, as JSON text.
    let (head, body) = get(served.addr, &format!("/projection.json?c={c}"));
    assert!(head.contains("Content-Type: application/json"));
    assert!(head.contains("X-Content-Type-Options: nosniff"));
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("JSON");
    assert_eq!(
        parsed["ledger"][0]["facets"]["reported_text"]
            .as_str()
            .expect("the reported text survives"),
        hostile,
        "the payload must survive semantically, not be sanitized away"
    );
    assert_eq!(
        parsed["ledger"][1]["tool_name"]
            .as_str()
            .expect("tool name"),
        hostile
    );
}

// ---------------------------------------------------------------------------
// The snapshot is taken once
// ---------------------------------------------------------------------------

#[test]
fn the_snapshot_is_immutable_and_the_source_file_is_never_re_read() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let path = dir.path().join("synthetic.ndjson");
    std::fs::write(&path, ordinary_recording()).expect("write the fixture");

    let snapshot = Snapshot::load(&path).expect("a valid recording should load");
    assert_eq!(snapshot.records(), 4);
    let served = serve(snapshot);
    let c = &served.capability;

    let (_, before) = get(served.addr, &format!("/projection.json?c={c}"));

    // Change the recording underneath the running viewer, in a way that would be
    // impossible to miss if anything re-read it.
    let longer = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_requested("toolu_later", "AddedAfterTheSnapshot"),
        ),
    ]);
    std::fs::write(&path, &longer).expect("rewrite the fixture");

    let (_, after) = get(served.addr, &format!("/projection.json?c={c}"));
    assert_eq!(before, after, "the served snapshot changed after a rewrite");
    assert!(!after.contains("AddedAfterTheSnapshot"));
    assert!(after.contains("toolu_a"));

    // Deleting it changes nothing either: nothing holds the file open.
    std::fs::remove_file(&path).expect("remove the fixture");
    let (head, after_delete) = get(served.addr, &format!("/projection.json?c={c}"));
    assert!(head.starts_with("HTTP/1.1 200 OK"));
    assert_eq!(before, after_delete);
}

#[test]
fn loading_a_recording_writes_nothing_and_leaves_it_byte_for_byte_unchanged() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let path = dir.path().join("synthetic.ndjson");
    let original = ordinary_recording();
    std::fs::write(&path, &original).expect("write the fixture");

    let served = serve(Snapshot::load(&path).expect("load"));
    let _ = get(
        served.addr,
        &format!("/projection.json?c={}", served.capability),
    );
    let _ = get(served.addr, &format!("/?c={}", served.capability));

    assert_eq!(
        std::fs::read_to_string(&path).expect("read back"),
        original,
        "the recording was modified"
    );
    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read the directory")
        .map(|entry| entry.expect("entry").file_name())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "the viewer created something beside the recording: {entries:?}"
    );
}

// ---------------------------------------------------------------------------
// Recording states
// ---------------------------------------------------------------------------

#[test]
fn a_corrupt_recording_fails_before_anything_is_bound_or_served() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let path = dir.path().join("corrupt.ndjson");
    std::fs::write(&path, "{\"schema_version\":2,\"nonsense\":true}\n").expect("write");

    let error = Snapshot::load(&path).expect_err("a corrupt recording must not load");
    let message = error.to_string();
    assert!(message.contains("line 1"), "{message}");
}

#[test]
fn a_missing_recording_path_is_an_error_and_no_directory_is_scanned() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    std::fs::write(dir.path().join("decoy.ndjson"), ordinary_recording()).expect("write");

    // A path that does not exist stays a path that does not exist. Nothing looks
    // in the directory beside it for something that would have worked.
    let missing = dir.path().join("absent.ndjson");
    Snapshot::load(&missing).expect_err("a missing recording must not load");

    // And a directory is not a recording.
    Snapshot::load(dir.path()).expect_err("a directory must not load as a recording");
}

#[test]
fn a_truncated_recording_is_served_with_its_truncation_carried_through() {
    let complete = ordinary_recording();
    let truncated = format!("{complete}{{\"schema_version\":2,\"session");
    let snapshot = snapshot_of(&truncated);
    assert!(snapshot.is_truncated());
    assert_eq!(snapshot.records(), 4);

    let served = serve(snapshot);
    let (head, body) = get(
        served.addr,
        &format!("/projection.json?c={}", served.capability),
    );
    assert!(head.starts_with("HTTP/1.1 200 OK"));
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("JSON");
    assert!(
        parsed["scope"]["valid_prefix"].is_object(),
        "truncation must reach the served projection: {}",
        parsed["scope"]
    );
    assert_eq!(parsed["scope"]["valid_prefix"]["records"], 4);
}

#[test]
fn an_empty_recording_serves_a_projection_that_declares_no_schema() {
    let served = serve(snapshot_of(""));
    let (head, body) = get(
        served.addr,
        &format!("/projection.json?c={}", served.capability),
    );
    assert!(head.starts_with("HTTP/1.1 200 OK"));
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("JSON");
    assert!(parsed["schema_version"].is_null());
    assert_eq!(parsed["records"].as_array().expect("records").len(), 0);
}

// ---------------------------------------------------------------------------
// The bundled assets
// ---------------------------------------------------------------------------

#[test]
fn the_bundled_assets_require_no_network_access() {
    let served = serve(snapshot_of(&ordinary_recording()));
    let c = &served.capability;
    let (_, page) = get(served.addr, &format!("/?c={c}"));
    let (_, stylesheet) = get(served.addr, &format!("/viewer.css?c={c}"));
    let (_, script) = get(served.addr, &format!("/viewer.js?c={c}"));

    for (name, asset) in [
        ("page", &page),
        ("stylesheet", &stylesheet),
        ("script", &script),
    ] {
        for remote in [
            "http://",
            "https://",
            "//cdn",
            "src=\"//",
            "@import url(http",
        ] {
            assert!(
                !asset.contains(remote),
                "the {name} references something remote: {remote:?}"
            );
        }
    }

    // Exactly one script element, external and same-origin. No inline script,
    // no worker, no storage, no analytics, no inline handler.
    assert_eq!(page.matches("<script").count(), 1);
    assert!(page.contains(&format!(
        "<script type=\"module\" src=\"/viewer.js?c={c}\"></script>"
    )));
    for forbidden in [
        "serviceWorker",
        "localStorage",
        "sessionStorage",
        "indexedDB",
        "onerror=",
        "onload=",
        "onclick=",
    ] {
        assert!(!page.contains(forbidden), "the page contains {forbidden:?}");
        assert!(
            !script.contains(forbidden),
            "the script contains {forbidden:?}"
        );
    }

    // The page's only stylesheet is same-origin and carries the capability.
    assert!(page.contains(&format!("href=\"/viewer.css?c={c}\"")));

    // It says what it is, including what it is not.
    assert!(page.contains("Not redacted"));
    assert!(page.contains("Rendering is not redacting"));
}

// ---------------------------------------------------------------------------
// Argument handling, at the level the library can see it
// ---------------------------------------------------------------------------

/// Start `witnessglass view --recording <path> --no-open` and return the child
/// plus the URL it printed. The caller kills it.
fn spawn_view(recording: &Path) -> (std::process::Child, String) {
    use std::io::BufRead;

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_witnessglass"))
        .args(["view", "--recording"])
        .arg(recording)
        .arg("--no-open")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the viewer should start");

    let stderr = child.stderr.take().expect("stderr is piped");
    let mut reader = std::io::BufReader::new(stderr);
    let mut url = None;
    for _ in 0..10 {
        let mut line = String::new();
        if reader.read_line(&mut line).expect("read stderr") == 0 {
            break;
        }
        if line.trim().starts_with("http://") {
            url = Some(line.trim().to_owned());
            break;
        }
    }
    let url = url.expect("the viewer should print its URL to stderr");

    // Keep draining stderr for the child's lifetime. Dropping the reader here
    // would close the read end of the pipe, and the viewer still has a line to
    // write after the URL — `eprintln!` to a closed pipe is an EPIPE panic, so
    // the viewer would die before `serve_forever`, and a request already
    // accepted onto the backlog would come back as a connection reset. That
    // race is narrow enough to pass locally and fail on a loaded CI runner.
    std::thread::spawn(move || {
        let mut drained = String::new();
        let _ = reader.read_to_string(&mut drained);
    });

    (child, url)
}

#[test]
fn the_command_serves_without_opening_a_browser_and_dies_with_the_process() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let path = dir.path().join("synthetic.ndjson");
    std::fs::write(&path, ordinary_recording()).expect("write the fixture");

    let (mut child, url) = spawn_view(&path);

    // The URL is loopback and carries a capability.
    assert!(url.starts_with("http://127.0.0.1:"), "{url}");
    let (rest, capability) = url
        .split_once("/?c=")
        .expect("the URL carries a capability");
    assert_eq!(capability.len(), 64);
    let addr: SocketAddr = rest
        .trim_start_matches("http://")
        .parse()
        .expect("the URL names a socket address");
    assert!(addr.ip().is_loopback());

    // It is genuinely serving, with `--no-open` having launched nothing.
    let (head, body) = get(addr, &format!("/projection.json?c={capability}"));
    assert!(head.starts_with("HTTP/1.1 200 OK"));
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("JSON");
    assert_eq!(parsed["records"].as_array().expect("records").len(), 4);

    // And it dies with the process, leaving no listener behind.
    child.kill().expect("the viewer should be killable");
    child.wait().expect("the viewer should exit");
    for _ in 0..50 {
        if TcpStream::connect(addr).is_err() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    panic!("the listener outlived the process that started it");
}

#[test]
fn the_command_refuses_a_recording_it_cannot_read_before_serving_anything() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let path = dir.path().join("corrupt.ndjson");
    std::fs::write(&path, "{\"schema_version\":2,\"nonsense\":true}\n").expect("write");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_witnessglass"))
        .args(["view", "--recording"])
        .arg(&path)
        .arg("--no-open")
        .output()
        .expect("the viewer should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("could not read"), "{stderr}");
    assert!(
        !stderr.contains("http://"),
        "no URL should be printed: {stderr}"
    );
}

#[test]
fn view_requires_a_recording_and_rejects_an_unknown_flag() {
    for args in [
        vec!["view"],
        vec!["view", "--no-open"],
        vec!["view", "--recording"],
        vec!["view", "--recording", "x.ndjson", "--host", "0.0.0.0"],
        vec!["view", "--recording", "x.ndjson", "--port", "8080"],
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_witnessglass"))
            .args(&args)
            .output()
            .expect("the CLI should run");
        assert!(
            !output.status.success(),
            "{args:?} should have been refused"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("http://"),
            "{args:?} printed a URL: {stderr}"
        );
    }
}

#[test]
fn nothing_in_the_serving_path_can_launch_a_browser() {
    // `--no-open` is the absence of a call rather than a flag threaded through
    // the server: binding and serving have no browser-launching code in them at
    // all, so there is no configuration under which serving opens a window.
    let source = std::fs::read_to_string(Path::new("src/view.rs")).expect("read src/view.rs");
    let (before_opener, opener_onwards) = source
        .split_once("pub fn open_in_browser")
        .expect("open_in_browser should exist");

    assert!(
        !before_opener.contains("Command::new"),
        "the binding and serving path must not spawn a process"
    );
    assert!(
        opener_onwards.contains("Command::new"),
        "opening a browser should be the one place a process is spawned"
    );

    // And nothing in the server binds anywhere but loopback.
    assert!(!source.contains("0.0.0.0"));
    assert!(!source.contains("UNSPECIFIED"));
    assert_eq!(source.matches("TcpListener::bind").count(), 1);
    assert!(source.contains("TcpListener::bind((Ipv4Addr::LOCALHOST, 0))"));
}
