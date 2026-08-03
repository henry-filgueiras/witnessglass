//! Minimal CLI over the recording kernel.
//!
//! Four verbs: append a structured event read from stdin, replay a recording,
//! translate one Claude Code command-hook payload into a recording, and view one
//! recording's projection in a browser. Nothing else is exposed, because nothing
//! else exists.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use witnessglass::claude::UnmodelledFields;
use witnessglass::view::{Snapshot, Viewer, open_in_browser};
use witnessglass::{Emission, Tail, append, replay_file};

const USAGE: &str = "\
witnessglass — a flight recorder for coding agents (experimental kernel)

USAGE:
    witnessglass append      --recording <PATH>
    witnessglass replay      --recording <PATH>
    witnessglass claude-hook --recordings-dir <DIR> [--strict-json-validation]
    witnessglass view        --recording <PATH> [--no-open]

    append        Read one JSON emission object from stdin and append it to the
                  recording as a complete record. Prints the written record.
    replay        Read a recording and print its records to stdout as NDJSON, in
                  canonical append order.
    claude-hook   Read exactly one Claude Code command-hook JSON object from
                  stdin and append its evidence to <DIR>/<session-id>.ndjson.
                  Prints nothing on success. Passive: it returns no decision,
                  no updated input or output, and no additional context, so it
                  cannot influence the session it records.

                  --strict-json-validation refuses any payload carrying a field
                  the adapter neither models nor deliberately drops, naming the
                  fields. A drift canary for one session, not a setting to leave
                  on: a refused payload is a record that was never written, and
                  the whole point of ignoring unknown fields is that an upstream
                  addition must not stop a recording mid-session. Also enabled by
                  WITNESSGLASS_STRICT_JSON=1, which is how you reach a hook that
                  Claude spawns from a settings file.
    view          Validate and project one recording, then serve that snapshot
                  read-only to a browser on a loopback port behind a per-launch
                  capability, and open it. Foreground and short-lived: it holds
                  the snapshot in memory, never re-reads the file, and dies with
                  this command. Not a daemon. --no-open serves without launching
                  a browser, which is what a remote terminal wants.

EXIT CODES:
    0  success
    1  error
    2  replay succeeded but the recording is incomplete (truncated tail)

    claude-hook only ever exits 0 or 1. Claude documents exit 1 as non-blocking
    for every hook this adapter supports, so a recorder failure is visible
    without interfering with the session.

A recording holds one session. It is not redacted and is not safe to share.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("witnessglass: {message}");
            ExitCode::from(1)
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let Some(command) = args.first() else {
        eprintln!("{USAGE}");
        return Ok(ExitCode::from(1));
    };

    match command.as_str() {
        "-h" | "--help" | "help" => {
            println!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        "-V" | "--version" => {
            println!("witnessglass {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        "append" => run_append(&flag_value(&args[1..], "--recording")?),
        "replay" => run_replay(&flag_value(&args[1..], "--recording")?),
        "claude-hook" => {
            let options = parse_claude_hook_args(&args[1..])?;
            run_claude_hook(&options.recordings_dir, options.unmodelled)
        }
        "view" => {
            let options = parse_view_args(&args[1..])?;
            run_view(&options)
        }
        other => Err(format!("unknown command {other:?}\n\n{USAGE}")),
    }
}

/// Environment equivalent of `--strict-json-validation`.
///
/// A flag would be the whole story if a human ran this command, but a hook is
/// spawned by Claude from a settings file, and `arm.sh` writes that file from a
/// fixed example. Without an environment path, arming a canary session means
/// hand-editing JSON that a script owns. `WITNESSGLASS_STRICT_JSON=1 claude`
/// reaches every hook the session spawns and reaches nothing else.
const STRICT_ENV: &str = "WITNESSGLASS_STRICT_JSON";

/// What `claude-hook` was asked to do.
struct ClaudeHookOptions {
    /// Directory the session's recording lives in.
    recordings_dir: PathBuf,
    /// What to do about payload fields the adapter has no model for.
    unmodelled: UnmodelledFields,
}

fn parse_claude_hook_args(args: &[String]) -> Result<ClaudeHookOptions, String> {
    let mut recordings_dir = None;
    // The flag wins when present; otherwise the environment decides. Any value
    // but the empty string enables it, so `=0` disabling it would be a trap —
    // unset it instead.
    let mut unmodelled = match std::env::var(STRICT_ENV) {
        Ok(value) if !value.is_empty() => UnmodelledFields::Reject,
        _ => UnmodelledFields::Ignore,
    };

    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--recordings-dir" => {
                let value = rest
                    .next()
                    .ok_or_else(|| "--recordings-dir requires a path".to_owned())?;
                recordings_dir = Some(PathBuf::from(value));
            }
            "--strict-json-validation" => unmodelled = UnmodelledFields::Reject,
            other => return Err(format!("unexpected argument {other:?}\n\n{USAGE}")),
        }
    }

    Ok(ClaudeHookOptions {
        recordings_dir: recordings_dir
            .ok_or_else(|| format!("--recordings-dir <DIR> is required\n\n{USAGE}"))?,
        unmodelled,
    })
}

/// What `view` was asked to do.
struct ViewOptions {
    /// The one recording to project. Taken exactly as given: nothing scans a
    /// directory, expands a glob, or picks a "latest" recording on the caller's
    /// behalf. A viewer that guesses which recording you meant is a viewer that
    /// can open the wrong one.
    recording: PathBuf,
    /// Whether to ask the desktop to open the URL.
    open: bool,
}

fn parse_view_args(args: &[String]) -> Result<ViewOptions, String> {
    let mut recording = None;
    let mut open = true;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--recording" => {
                let value = rest
                    .next()
                    .ok_or_else(|| "--recording requires a path".to_owned())?;
                recording = Some(PathBuf::from(value));
            }
            "--no-open" => open = false,
            other => return Err(format!("unexpected argument {other:?}\n\n{USAGE}")),
        }
    }
    let recording =
        recording.ok_or_else(|| format!("--recording <PATH> is required\n\n{USAGE}"))?;
    Ok(ViewOptions { recording, open })
}

/// Serve one recording's projection to a browser, in the foreground, until the
/// process is ended.
///
/// The order matters and is part of the contract: replay, project, and bind
/// happen before a browser is launched, so a corrupt recording fails at a
/// terminal rather than in a tab. Nothing about the request stream is logged —
/// the URL carries the capability and the responses carry evidence, so the only
/// safe amount of request logging is none.
fn run_view(options: &ViewOptions) -> Result<ExitCode, String> {
    let snapshot = Snapshot::load(&options.recording)
        .map_err(|err| format!("could not read {}: {err}", options.recording.display()))?;

    let records = snapshot.records();
    let version = match snapshot.schema_version() {
        Some(version) => format!("schema v{version}"),
        None => "no records, so no schema version".to_owned(),
    };
    let truncated = snapshot.is_truncated();

    let viewer = Viewer::bind(snapshot).map_err(|err| format!("could not bind: {err}"))?;
    let url = viewer
        .url()
        .map_err(|err| format!("could not read the bound address: {err}"))?;

    eprintln!("projected {records} record(s) ({version}); serving one immutable snapshot");
    if truncated {
        eprintln!(
            "INCOMPLETE: this recording ends mid-record. The valid prefix is served and every \
             absence in the projection is scoped to that prefix, not to a complete recording."
        );
    }
    eprintln!("this snapshot is NOT redacted and is not safe to share; rendering is not redacting");
    // Printed before any launch attempt, so the URL is available whether or not
    // opening works.
    eprintln!("{url}");

    if options.open
        && let Err(err) = open_in_browser(&url)
    {
        eprintln!("could not open a browser ({err}); open the URL above yourself");
    }

    eprintln!("serving in the foreground; press Ctrl-C to stop");
    viewer
        .serve_forever()
        .map_err(|err| format!("stopped serving: {err}"))?;
    Ok(ExitCode::SUCCESS)
}

/// Parse the one flag every command takes.
fn flag_value(args: &[String], name: &'static str) -> Result<PathBuf, String> {
    let mut path = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            arg if arg == name => {
                let value = rest
                    .next()
                    .ok_or_else(|| format!("{name} requires a path"))?;
                path = Some(PathBuf::from(value));
            }
            other => return Err(format!("unexpected argument {other:?}\n\n{USAGE}")),
        }
    }
    path.ok_or_else(|| format!("{name} <PATH> is required\n\n{USAGE}"))
}

fn read_stdin() -> Result<String, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|err| format!("could not read stdin: {err}"))?;
    Ok(input)
}

fn run_append(recording: &Path) -> Result<ExitCode, String> {
    let input = read_stdin()?;

    let emission: Emission = serde_json::from_str(input.trim())
        .map_err(|err| format!("could not parse emission from stdin: {err}"))?;

    let record = append(recording, &emission, jiff::Timestamp::now())
        .map_err(|err| format!("could not append: {err}"))?;

    let line = serde_json::to_string(&record)
        .map_err(|err| format!("could not render written record: {err}"))?;
    println!("{line}");
    Ok(ExitCode::SUCCESS)
}

/// Translate one Claude Code command hook and append what it delivered.
///
/// Silent on success by design. Claude reads a hook's stdout for decisions,
/// updated tool input and output, and additional context; writing nothing there
/// is what makes this adapter incapable of influencing the session. Failures go
/// to stderr with exit 1, which Claude documents as non-blocking for every hook
/// configured here — the recording stops, the session does not.
fn run_claude_hook(
    recordings_dir: &Path,
    unmodelled: UnmodelledFields,
) -> Result<ExitCode, String> {
    let input = read_stdin()?;

    let translation = witnessglass::claude::translate_with(&input, unmodelled)
        .map_err(|err| format!("could not translate Claude hook: {err}"))?;

    std::fs::create_dir_all(recordings_dir).map_err(|err| {
        format!(
            "could not create recordings directory {}: {err}",
            recordings_dir.display()
        )
    })?;

    // `file_name` is validated to be a single path component with no separator
    // and no `.` component, so this join cannot leave the recordings directory.
    let recording = recordings_dir.join(&translation.file_name);

    for emission in &translation.emissions {
        append(&recording, emission, jiff::Timestamp::now()).map_err(|err| {
            format!(
                "could not append {} to {}: {err}",
                emission.event.kind(),
                recording.display()
            )
        })?;
    }

    Ok(ExitCode::SUCCESS)
}

fn run_replay(recording: &Path) -> Result<ExitCode, String> {
    let replay = replay_file(recording).map_err(|err| format!("could not replay: {err}"))?;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for record in &replay.records {
        let line = serde_json::to_string(record)
            .map_err(|err| format!("could not render record: {err}"))?;
        writeln!(out, "{line}").map_err(|err| format!("could not write record: {err}"))?;
    }
    out.flush()
        .map_err(|err| format!("could not flush stdout: {err}"))?;

    let version = match replay.schema_version {
        Some(version) => format!("schema v{version}"),
        None => "no records, so no schema version".to_owned(),
    };

    match replay.tail {
        Tail::Complete => {
            eprintln!(
                "replayed {} record(s) in append order ({version}); recording is complete",
                replay.records.len()
            );
            Ok(ExitCode::SUCCESS)
        }
        Tail::Truncated { byte_offset, bytes } => {
            eprintln!(
                "replayed {} record(s) in append order ({version}); INCOMPLETE: recording ends \
                 with a {bytes}-byte unterminated fragment at byte {byte_offset}, which is not \
                 an event and has not been replayed",
                replay.records.len()
            );
            Ok(ExitCode::from(2))
        }
    }
}
