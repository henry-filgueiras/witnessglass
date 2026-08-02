//! Minimal CLI over the recording kernel.
//!
//! Two verbs, one flag each: append a structured event read from stdin, and
//! replay a recording. Nothing else is exposed, because nothing else exists.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use witnessglass::{Emission, Tail, append, replay_file};

const USAGE: &str = "\
witnessglass — a flight recorder for coding agents (experimental kernel)

USAGE:
    witnessglass append --recording <PATH>
    witnessglass replay --recording <PATH>

    append   Read one JSON emission object from stdin and append it to the
             recording as a complete record. Prints the written record.
    replay   Read a recording and print its records to stdout as NDJSON, in
             canonical append order.

EXIT CODES:
    0  success
    1  error
    2  replay succeeded but the recording is incomplete (truncated tail)

A recording holds one session. It is not redacted and is not safe to share.
No agent adapter exists yet; events come from whoever runs this command.";

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
        "append" => run_append(&recording_path(&args[1..])?),
        "replay" => run_replay(&recording_path(&args[1..])?),
        other => Err(format!("unknown command {other:?}\n\n{USAGE}")),
    }
}

/// Parse the one flag every command takes.
fn recording_path(args: &[String]) -> Result<PathBuf, String> {
    let mut path = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--recording" => {
                let value = rest
                    .next()
                    .ok_or_else(|| "--recording requires a path".to_owned())?;
                path = Some(PathBuf::from(value));
            }
            other => return Err(format!("unexpected argument {other:?}\n\n{USAGE}")),
        }
    }
    path.ok_or_else(|| format!("--recording <PATH> is required\n\n{USAGE}"))
}

fn run_append(recording: &std::path::Path) -> Result<ExitCode, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|err| format!("could not read stdin: {err}"))?;

    let emission: Emission = serde_json::from_str(input.trim())
        .map_err(|err| format!("could not parse emission from stdin: {err}"))?;

    let record = append(recording, &emission, jiff::Timestamp::now())
        .map_err(|err| format!("could not append: {err}"))?;

    let line = serde_json::to_string(&record)
        .map_err(|err| format!("could not render written record: {err}"))?;
    println!("{line}");
    Ok(ExitCode::SUCCESS)
}

fn run_replay(recording: &std::path::Path) -> Result<ExitCode, String> {
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

    match replay.tail {
        Tail::Complete => {
            eprintln!(
                "replayed {} record(s) in append order; recording is complete",
                replay.records.len()
            );
            Ok(ExitCode::SUCCESS)
        }
        Tail::Truncated { byte_offset, bytes } => {
            eprintln!(
                "replayed {} record(s) in append order; INCOMPLETE: recording ends with a \
                 {bytes}-byte unterminated fragment at byte {byte_offset}, which is not an \
                 event and has not been replayed",
                replay.records.len()
            );
            Ok(ExitCode::from(2))
        }
    }
}
