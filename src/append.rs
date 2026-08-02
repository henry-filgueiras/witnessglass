//! Appending a record to a recording.
//!
//! # The append transaction
//!
//! Claude Code runs hooks as separate, short-lived, parallel processes, so two
//! emitters genuinely can race for the same recording. Deciding the next
//! sequence number and writing the record must therefore be one indivisible
//! step: if they were separate, two processes could read the same last sequence
//! and both claim it.
//!
//! The transaction is serialized with an advisory exclusive lock taken on the
//! recording file itself (`std::fs::File::lock`), held across the whole
//! read-tail / decide-sequence / write-record sequence and released when the
//! handle drops. The file is opened in append mode, so the write lands at the
//! end regardless of where reading left the cursor.
//!
//! This needs no daemon, no database, and no shared process. It does assume
//! every writer is a WitnessGlass appender honoring the same advisory lock; a
//! foreign process writing to the file concurrently is out of scope.
//!
//! # What durability is actually claimed
//!
//! A successful append means the record was written and `sync_data` returned:
//! the bytes were handed to the operating system and it reported them flushed.
//! It does not claim the write is atomic against power loss. A crash mid-write
//! can still leave an unterminated fragment — which is precisely why a
//! truncated tail is a first-class, readable state rather than an error.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::error::{Error, Result};
use crate::record::v2::{Emission, Record, SCHEMA_VERSION};
use crate::record::{AnyRecord, parse_line};

/// Size of the backward-scan window used to locate the final record.
const TAIL_CHUNK: u64 = 8192;

/// Append one schema v2 emission to `recording`, creating it if absent.
///
/// Only the current schema version is writable. A recording written under an
/// earlier version stays replayable and is refused here, because one recording
/// uses one vocabulary throughout.
///
/// `recorded_at` is stamped into the record as descriptive metadata. It is
/// injected rather than read from the clock inside so that tests can exercise
/// equal and backward-moving timestamps; it has no effect on ordering.
///
/// Returns the record as written.
pub fn append(
    recording: &Path,
    emission: &Emission,
    recorded_at: jiff::Timestamp,
) -> Result<Record> {
    emission.validate()?;

    let mut file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(recording)?;

    // Everything from here until the handle drops is the serialized
    // transaction. Take the lock before measuring the file: another appender
    // may be mid-write.
    file.lock()?;

    let len = file.metadata()?.len();
    let previous = read_final_record(&mut file, len)?;

    let sequence = match &previous {
        Some(record) => {
            if record.schema_version() != SCHEMA_VERSION {
                return Err(Error::AppendVersionMismatch {
                    recording: record.schema_version(),
                    writing: SCHEMA_VERSION,
                });
            }
            if record.session_id() != emission.session_id {
                return Err(Error::EmissionSessionMismatch {
                    recording: record.session_id().to_owned(),
                    emission: emission.session_id.clone(),
                });
            }
            // Checked, because a wrapped sequence is the one failure the
            // ordering contract cannot survive: it would silently restart the
            // canonical chain at zero rather than fail.
            record
                .sequence()
                .checked_add(1)
                .ok_or(Error::SequenceExhausted {
                    last: record.sequence(),
                })?
        }
        None => 1,
    };

    let record = Record {
        schema_version: SCHEMA_VERSION,
        session_id: emission.session_id.clone(),
        sequence,
        recorded_at,
        context: emission.context.clone(),
        provenance: emission.provenance.clone(),
        event: emission.event.clone(),
    };

    let mut line = serde_json::to_string(&record).map_err(Error::Serialize)?;
    line.push('\n');

    // One complete record, written whole while the lock is held.
    file.write_all(line.as_bytes())?;
    file.flush()?;
    file.sync_data()?;

    Ok(record)
}

/// Read and parse the recording's last complete record.
///
/// Returns `None` for an empty recording. Refuses outright if the recording
/// ends with an unterminated fragment: appending would splice a new record onto
/// a partial one and manufacture a corrupt line out of two honest halves.
fn read_final_record(file: &mut File, len: u64) -> Result<Option<AnyRecord>> {
    if len == 0 {
        return Ok(None);
    }

    file.seek(SeekFrom::Start(len - 1))?;
    let mut last = [0u8; 1];
    file.read_exact(&mut last)?;
    if last[0] != b'\n' {
        let fragment = trailing_fragment_len(file, len)?;
        return Err(Error::AppendAfterTruncation { bytes: fragment });
    }

    let start = final_line_start(file, len)?;
    let mut bytes = vec![0u8; (len - 1 - start) as usize];
    file.seek(SeekFrom::Start(start))?;
    file.read_exact(&mut bytes)?;

    let line = std::str::from_utf8(&bytes).map_err(|err| Error::Corruption {
        line: 0,
        reason: format!("final record is not valid UTF-8: {err}"),
    })?;

    Ok(Some(parse_line(line, 0)?))
}

/// Byte offset just past the newline that ends the second-to-last record, i.e.
/// where the final complete record begins. Assumes the file ends with `\n`.
fn final_line_start(file: &mut File, len: u64) -> Result<u64> {
    // Exclude the terminating newline from the search.
    let mut end = len - 1;
    while end > 0 {
        let start = end.saturating_sub(TAIL_CHUNK);
        let mut chunk = vec![0u8; (end - start) as usize];
        file.seek(SeekFrom::Start(start))?;
        file.read_exact(&mut chunk)?;
        if let Some(index) = chunk.iter().rposition(|&byte| byte == b'\n') {
            return Ok(start + index as u64 + 1);
        }
        end = start;
    }
    Ok(0)
}

/// Length of the unterminated fragment at the end of the file.
fn trailing_fragment_len(file: &mut File, len: u64) -> Result<usize> {
    let mut end = len;
    while end > 0 {
        let start = end.saturating_sub(TAIL_CHUNK);
        let mut chunk = vec![0u8; (end - start) as usize];
        file.seek(SeekFrom::Start(start))?;
        file.read_exact(&mut chunk)?;
        if let Some(index) = chunk.iter().rposition(|&byte| byte == b'\n') {
            return Ok((len - (start + index as u64 + 1)) as usize);
        }
        end = start;
    }
    Ok(len as usize)
}
