//! Minimal CLI over the recording kernel.
//!
//! Three verbs: append a structured event read from stdin, replay a recording,
//! and translate one Claude Code command-hook payload into a recording. Nothing
//! else is exposed, because nothing else exists.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use witnessglass::{Emission, Tail, append, replay_file};

const USAGE: &str = "\
witnessglass — a flight recorder for coding agents (experimental kernel)

USAGE:
    witnessglass append      --recording <PATH>
    witnessglass replay      --recording <PATH>
    witnessglass claude-hook --recordings-dir <DIR>

    append        Read one JSON emission object from stdin and append it to the
                  recording as a complete record. Prints the written record.
    replay        Read a recording and print its records to stdout as NDJSON, in
                  canonical append order.
    claude-hook   Read exactly one Claude Code command-hook JSON object from
                  stdin and append its evidence to <DIR>/<session-id>.ndjson.
                  Prints nothing on success. Passive: it returns no decision,
                  no updated input or output, and no additional context, so it
                  cannot influence the session it records.

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
        "claude-hook" => run_claude_hook(&flag_value(&args[1..], "--recordings-dir")?),
        other => Err(format!("unknown command {other:?}\n\n{USAGE}")),
    }
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
fn run_claude_hook(recordings_dir: &Path) -> Result<ExitCode, String> {
    let input = read_stdin()?;

    let translation = witnessglass::claude::translate(&input)
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
