//! A deterministic synthetic recording whose hidden structure is known in
//! advance.
//!
//! **Disposable.** See [`crate::experiment`].
//!
//! # What this is for
//!
//! A substrate that projects a recording into numbers can be wrong in ways that
//! look right. If the only recordings available are real ones, there is nothing
//! to check the projection against except the projection's own plausibility,
//! which is how sprint:3's rationale describes a system asking itself whether it
//! is complete.
//!
//! So the structure is decided first, in the constants below, and the recording
//! is generated from them. `tests/behavioral_signal.rs` then asserts that the
//! projection recovers the structure the constants declare. The fixture is
//! committed at `fixtures/synthetic-behavioral-oracle.ndjson` and
//! [`ndjson`] regenerates it byte for byte, so it cannot drift from the
//! structure it claims to hold.
//!
//! # The structure, in milliseconds from the origin
//!
//! ```text
//!       0 .. 60000   baseline    one two-record call every 6 s, one tool name
//!   60000 .. 90000   motif       a four-call, nine-record figure every 8 s
//!   90000 ..150000   baseline    identical in shape to the first
//!  150000            regime change
//!  150000 ..210000   elevated    a two-record call every 1.5 s, two tool names,
//!                                larger recorded responses, one subagent pair,
//!                                and no reported intent at all
//!  210000 ..240000   motif again the same figure, with deterministic noise:
//!                                jittered start times, jittered offsets, and
//!                                one call that fails instead of succeeding
//!  240000            session ends
//! ```
//!
//! The regime change at 150000 is deliberately multivariate: the rate rises, the
//! set of delivered tool names changes, the recorded response sizes grow, and the
//! `reported` channel goes silent. A projection that recovers only one of those
//! four has lost something.
//!
//! # What it is built from
//!
//! Only the v2 vocabulary as it actually is. Every record is one this build can
//! write, replay, and project through the ordinary path — no field is invented,
//! stretched, or given a meaning the schema does not have. The one thing it does
//! not reproduce is a real integration's timing, which is the point: these
//! timestamps are chosen, and a real recording's are not.
//!
//! # Synthetic, and obviously so
//!
//! The session id, every tool name, every path, every command, and every id
//! contains the string `synthetic`. There is no real path, host, repository, or
//! command anywhere in it, and `tests/behavioral_signal.rs` asserts that per
//! line. `CLAUDE.md` §5 requires fixtures to be synthetic and obviously so; this
//! one is generated, so the requirement is a property of the generator rather
//! than of whoever last edited a file by hand.
//!
//! # Two fixtures, and the difference between them
//!
//! [`records`] and [`ndjson`] build the **legible** oracle described above. It is
//! deliberately a best case: dense enough to read a bucket dump by eye, with
//! structure placed where it is easy to see. sprint:4 measured it at 78% empty at
//! 500 ms and measured a real session at 94% empty, so the legible oracle is
//! roughly four times denser than reality and flatters anything run over it.
//!
//! [`sparse`] builds the **stress case** sprint:5 added for that reason: the same
//! kinds of structure at a density in the band the real recording established.
//! Neither replaces the other. A result that holds on the legible oracle and
//! fails on the sparse one is a result about legibility, and the pair exists so
//! that difference is measurable rather than assumed.
//!
//! Neither fixture is derived from a real recording. The sparse one takes one
//! number from observation — roughly how empty a real session's buckets are — and
//! nothing else: no content, no timing, no tool name, no payload.

use crate::record::{Channel, Provenance, v2};

/// Session id of the oracle recording.
pub const SESSION_ID: &str = "sess-synthetic-behavioral-oracle";

/// Adapter name stamped on every record.
pub const ADAPTER: &str = "synthetic-oracle-adapter";

/// Origin timestamp, as RFC 3339. Fixed, so the fixture is reproducible.
pub const ORIGIN: &str = "2026-02-01T00:00:00Z";

/// End of the first baseline, and start of the first motif, in milliseconds.
pub const FIRST_MOTIF_START_MS: u64 = 60_000;
/// End of the first motif, in milliseconds.
pub const FIRST_MOTIF_END_MS: u64 = 90_000;
/// Period between motif instances, in milliseconds.
pub const MOTIF_PERIOD_MS: u64 = 8_000;
/// Records one motif instance contributes.
pub const MOTIF_RECORDS_PER_INSTANCE: usize = 9;
/// Motif instances in each motif interval.
pub const MOTIF_INSTANCES: u64 = 4;

/// The regime change, in milliseconds. Also the start of the elevated regime.
pub const REGIME_CHANGE_MS: u64 = 150_000;
/// End of the elevated regime, in milliseconds.
pub const ELEVATED_END_MS: u64 = 210_000;
/// Period between calls in the elevated regime, in milliseconds.
pub const ELEVATED_PERIOD_MS: u64 = 1_500;

/// Start of the motif's recurrence, in milliseconds.
pub const SECOND_MOTIF_START_MS: u64 = ELEVATED_END_MS;
/// End of the recurrence, and the session boundary, in milliseconds.
pub const SESSION_END_MS: u64 = 240_000;

/// Period between baseline calls, in milliseconds.
pub const BASELINE_PERIOD_MS: u64 = 6_000;

/// Tool name delivered by the baseline and by the motif's first call.
pub const TOOL_READER: &str = "SyntheticReader";
/// Tool name delivered by the motif's second call.
pub const TOOL_SEARCHER: &str = "SyntheticSearcher";
/// Tool name delivered by the motif's third call and by the elevated regime.
pub const TOOL_EDITOR: &str = "SyntheticEditor";
/// Tool name delivered by the motif's fourth call and by the elevated regime.
pub const TOOL_SHELL: &str = "SyntheticShell";

/// Every tool name the oracle delivers, in the order it first delivers them.
pub const TOOL_NAMES: [&str; 4] = [TOOL_READER, TOOL_SEARCHER, TOOL_EDITOR, TOOL_SHELL];

/// The oracle recording, as complete v2 records in canonical append order.
///
/// Deterministic and clock-free: sequence numbers are assigned by position and
/// every timestamp is the origin plus a computed offset.
pub fn records() -> Vec<v2::Record> {
    let mut builder = Builder::new(Naming {
        session_id: SESSION_ID,
        adapter: ADAPTER,
        prompt_id: "prompt-synthetic-oracle-0001",
        call_prefix: "toolu_synthetic_oracle_",
        origin: ORIGIN,
    });

    builder.session_started(0);

    baseline(&mut builder, 3_000, FIRST_MOTIF_START_MS);
    motif(&mut builder, FIRST_MOTIF_START_MS, Noise::None);
    baseline(&mut builder, FIRST_MOTIF_END_MS + 3_000, REGIME_CHANGE_MS);
    elevated(&mut builder);
    motif(&mut builder, SECOND_MOTIF_START_MS, Noise::Deterministic);

    builder.session_ended(SESSION_END_MS);
    builder.records
}

/// The oracle recording, as the NDJSON bytes of a complete recording.
///
/// One newline-terminated line per record, which is what
/// `fixtures/synthetic-behavioral-oracle.ndjson` holds.
pub fn ndjson() -> String {
    let mut out = String::new();
    for record in records() {
        // Serializing a record this module built cannot fail; the arm keeps the
        // function total without fabricating a line.
        if let Ok(line) = serde_json::to_string(&record) {
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}

/// Sparse baseline: one call every [`BASELINE_PERIOD_MS`], one tool name, small
/// recorded responses, no reported intent.
fn baseline(builder: &mut Builder, from_ms: u64, until_ms: u64) {
    let mut at = from_ms;
    while at < until_ms {
        let id = builder.next_call_id();
        builder.tool_requested(
            at,
            TOOL_READER,
            &id,
            serde_json::json!({ "path": "/synthetic/tree/baseline.txt" }),
        );
        builder.tool_succeeded(
            at + 120,
            TOOL_READER,
            &id,
            serde_json::json!({ "path": "/synthetic/tree/baseline.txt" }),
            serde_json::json!({ "bytes": 64, "text": "synthetic baseline content" }),
        );
        at += BASELINE_PERIOD_MS;
    }
}

/// Whether a motif instance is the clean original or the noisy recurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Noise {
    /// Exact periodicity, exact offsets, every call succeeds.
    None,
    /// Jittered start times and offsets, and one failing call. Every value comes
    /// from the fixed generator below, so "noise" here is reproducible.
    Deterministic,
}

/// The motif: a reported intent followed by four calls in a fixed tool order,
/// repeated [`MOTIF_INSTANCES`] times at [`MOTIF_PERIOD_MS`].
fn motif(builder: &mut Builder, start_ms: u64, noise: Noise) {
    // A fixed 64-bit LCG rather than a random source: the recurrence has to be
    // reproducible to be an oracle, and "modest deterministic noise" is the
    // brief. Seeded from the interval so the two motif intervals differ.
    let mut jitter = Lcg::new(0x5157_4954_4E45_5353 ^ start_ms);

    for instance in 0..MOTIF_INSTANCES {
        let nominal = start_ms + instance * MOTIF_PERIOD_MS;
        let at = match noise {
            Noise::None => nominal,
            Noise::Deterministic => nominal + jitter.next_below(700),
        };
        let skew = |offset: u64, jitter: &mut Lcg| match noise {
            Noise::None => at + offset,
            Noise::Deterministic => at + offset + jitter.next_below(150),
        };

        let intent_id = builder.peek_call_id();
        builder.reported_intent(
            at,
            &intent_id,
            "run the synthetic motif over the synthetic tree",
        );

        for (index, (tool, request_offset, response_offset)) in [
            (TOOL_READER, 100, 300),
            (TOOL_SEARCHER, 600, 900),
            (TOOL_EDITOR, 1_200, 1_500),
            (TOOL_SHELL, 1_800, 2_200),
        ]
        .into_iter()
        .enumerate()
        {
            let id = builder.next_call_id();
            let input = serde_json::json!({
                "target": "/synthetic/tree/motif",
                "step": index,
            });
            builder.tool_requested(skew(request_offset, &mut jitter), tool, &id, input.clone());

            // The recurrence's one structural deviation: its third instance's
            // shell call fails instead of succeeding. A motif that repeats
            // perfectly is not a test of anything that has to tolerate noise.
            let failing = noise == Noise::Deterministic && instance == 2 && tool == TOOL_SHELL;
            if failing {
                builder.tool_failed(
                    skew(response_offset, &mut jitter),
                    tool,
                    &id,
                    input,
                    "synthetic failure injected by the oracle fixture",
                );
            } else {
                builder.tool_succeeded(
                    skew(response_offset, &mut jitter),
                    tool,
                    &id,
                    input,
                    serde_json::json!({ "matches": 3, "note": "synthetic motif result" }),
                );
            }
        }
    }
}

/// The elevated regime: a faster call rate, a different pair of tool names,
/// larger recorded responses, one subagent pair, and no reported intent.
fn elevated(builder: &mut Builder) {
    let mut at = REGIME_CHANGE_MS;
    let mut index = 0u64;
    let mut subagent_started = false;
    let mut subagent_stopped = false;

    while at < ELEVATED_END_MS {
        if !subagent_started && at >= 170_000 {
            builder.subagent_started(at, "agent-synthetic-oracle-child");
            subagent_started = true;
        }
        if !subagent_stopped && at >= 200_000 {
            builder.subagent_stopped(at, "agent-synthetic-oracle-child");
            subagent_stopped = true;
        }

        let tool = if index.is_multiple_of(2) {
            TOOL_SHELL
        } else {
            TOOL_EDITOR
        };
        let id = builder.next_call_id();
        let input = serde_json::json!({
            "command": "synthetic-build --target /synthetic/tree/elevated",
            "step": index,
        });
        builder.tool_requested(at, tool, &id, input.clone());
        builder.tool_succeeded(
            at + 400,
            tool,
            &id,
            input,
            // Deliberately larger than the baseline's response, so the recorded
            // response-size dimension separates the regimes on magnitude as well
            // as on rate.
            serde_json::json!({
                "exit_status": 0,
                "stdout": "synthetic elevated output line ".repeat(12),
                "stderr": "",
            }),
        );

        index += 1;
        at += ELEVATED_PERIOD_MS;
    }
}

/// A fixed linear congruential generator, so "noise" is reproducible.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_below(&mut self, bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % bound
    }
}

/// What a builder stamps on every record it assembles.
///
/// Parameterized because sprint:5 added a second fixture, and two generators
/// sharing one builder is better than two copies of the record assembly drifting
/// apart. The legible fixture's values are unchanged, and the test that
/// regenerates it byte for byte is what says so.
struct Naming {
    session_id: &'static str,
    adapter: &'static str,
    prompt_id: &'static str,
    call_prefix: &'static str,
    origin: &'static str,
}

/// Assembles records in canonical order, owning the sequence counter and the
/// timestamp arithmetic so no caller can get either wrong.
struct Builder {
    records: Vec<v2::Record>,
    naming: Naming,
    origin: jiff::Timestamp,
    calls: u64,
}

impl Builder {
    fn new(naming: Naming) -> Self {
        Self {
            records: Vec::new(),
            // The constant is a valid RFC 3339 timestamp, checked by the test
            // that regenerates the fixture; `unwrap_or` keeps the builder total
            // without a panic path.
            origin: naming
                .origin
                .parse::<jiff::Timestamp>()
                .unwrap_or(jiff::Timestamp::UNIX_EPOCH),
            naming,
            calls: 0,
        }
    }

    fn at(&self, offset_ms: u64) -> jiff::Timestamp {
        self.origin + jiff::SignedDuration::from_millis(offset_ms as i64)
    }

    fn next_call_id(&mut self) -> String {
        self.calls += 1;
        format!("{}{:04}", self.naming.call_prefix, self.calls)
    }

    /// The id the next call will use, so a reported intent can cite the call it
    /// is about before that call's records exist. Correlation, not fusion: the
    /// intent stays a separate record on the `reported` channel.
    fn peek_call_id(&self) -> String {
        format!("{}{:04}", self.naming.call_prefix, self.calls + 1)
    }

    fn push(&mut self, offset_ms: u64, channel: Channel, mechanism: &str, event: v2::Event) {
        self.records.push(v2::Record {
            schema_version: v2::SCHEMA_VERSION,
            session_id: self.naming.session_id.to_owned(),
            sequence: self.records.len() as u64 + 1,
            recorded_at: self.at(offset_ms),
            context: v2::Context {
                prompt_id: Some(self.naming.prompt_id.to_owned()),
                ..v2::Context::default()
            },
            provenance: Provenance {
                channel,
                adapter: self.naming.adapter.to_owned(),
                mechanism: mechanism.to_owned(),
            },
            event,
        });
    }

    fn session_started(&mut self, offset_ms: u64) {
        self.push(
            offset_ms,
            Channel::Recorder,
            "synthetic:SessionStart",
            v2::Event::SessionStarted(v2::SessionStarted {
                source: Some("startup".to_owned()),
            }),
        );
    }

    fn session_ended(&mut self, offset_ms: u64) {
        self.push(
            offset_ms,
            Channel::Recorder,
            "synthetic:SessionEnd",
            v2::Event::SessionEnded(v2::SessionEnded {
                reason: Some("clear".to_owned()),
            }),
        );
    }

    fn reported_intent(&mut self, offset_ms: u64, tool_use_id: &str, text: &str) {
        self.push(
            offset_ms,
            Channel::Reported,
            "synthetic:PreToolUse#tool_input.description",
            v2::Event::ReportedIntent(v2::ReportedIntent {
                text: text.to_owned(),
                tool_use_id: Some(tool_use_id.to_owned()),
            }),
        );
    }

    fn tool_requested(
        &mut self,
        offset_ms: u64,
        tool_name: &str,
        tool_use_id: &str,
        requested_input: serde_json::Value,
    ) {
        self.push(
            offset_ms,
            Channel::Observed,
            "synthetic:PreToolUse",
            v2::Event::ToolRequested(v2::ToolRequested {
                tool_use_id: tool_use_id.to_owned(),
                tool_name: tool_name.to_owned(),
                requested_input,
            }),
        );
    }

    fn tool_succeeded(
        &mut self,
        offset_ms: u64,
        tool_name: &str,
        tool_use_id: &str,
        effective_input: serde_json::Value,
        response: serde_json::Value,
    ) {
        self.push(
            offset_ms,
            Channel::Observed,
            "synthetic:PostToolUse",
            v2::Event::ToolSucceeded(v2::ToolSucceeded {
                tool_use_id: tool_use_id.to_owned(),
                tool_name: tool_name.to_owned(),
                effective_input,
                response,
                duration_ms: None,
            }),
        );
    }

    fn tool_failed(
        &mut self,
        offset_ms: u64,
        tool_name: &str,
        tool_use_id: &str,
        effective_input: serde_json::Value,
        error: &str,
    ) {
        self.push(
            offset_ms,
            Channel::Observed,
            "synthetic:PostToolUseFailure",
            v2::Event::ToolFailed(v2::ToolFailed {
                tool_use_id: tool_use_id.to_owned(),
                tool_name: tool_name.to_owned(),
                effective_input,
                error: error.to_owned(),
                interrupted: None,
                duration_ms: None,
            }),
        );
    }

    fn subagent_started(&mut self, offset_ms: u64, agent_id: &str) {
        self.push(
            offset_ms,
            Channel::Observed,
            "synthetic:SubagentStart",
            v2::Event::SubagentStarted(v2::SubagentStarted {
                agent_id: agent_id.to_owned(),
                agent_type: Some("synthetic-worker".to_owned()),
                // Never supplied, because the integration this imitates does not
                // supply it. A fixture that invented parentage would be teaching
                // the projection something untrue.
                parent_agent_id: None,
                parent_agent_type: None,
            }),
        );
    }

    fn subagent_stopped(&mut self, offset_ms: u64, agent_id: &str) {
        self.push(
            offset_ms,
            Channel::Observed,
            "synthetic:SubagentStop",
            v2::Event::SubagentStopped(v2::SubagentStopped {
                agent_id: agent_id.to_owned(),
                agent_type: Some("synthetic-worker".to_owned()),
                parent_agent_id: None,
                parent_agent_type: None,
            }),
        );
    }
}

/// The sparse companion oracle: the same kinds of structure at a density
/// informed by a real recording.
///
/// **Disposable.** sprint:5, task:15.
///
/// # Why a second fixture
///
/// sprint:4 measured the legible oracle at 78% empty at 500 ms and a real
/// 234-record session at 94% empty. A detector validated only against the
/// legible one has been validated against a best case that reality does not
/// supply. This fixture exists to be the stress case, and its only inheritance
/// from observation is a target density — no content, no timing, no tool name,
/// and no payload is derived from any real recording.
///
/// # The structure, in milliseconds from the origin
///
/// ```text
///        0 ..  300000   sparse baseline   one two-record call every 30 s
///   300000 ..  540000   motif             a five-record figure every 8 s, exactly
///   540000 ..  780000   sparse baseline   as before
///   780000 .. 1080000   regime block      one two-record call every 6 s, a
///                                         different tool name throughout, and one
///                                         subagent pair inside it
///  1080000 .. 1200000   recurrence        the same figure with deterministic
///                                         jitter, and one call that fails
///  1200000             session ends
/// ```
///
/// # Two structures, on purpose one dyadic and one not
///
/// The motif period is 8 s, which is 16 base samples at 500 ms and therefore a
/// power of two: a Haar transform sees it cleanly, and it is the positive
/// control. The regime block is 300 s, which is 600 base samples and sits
/// between 2^9 and 2^10: a Haar transform must smear it across two levels. A
/// fixture in which every structure happened to be dyadic would be a fixture
/// built to make one transform look good.
pub mod sparse {
    use super::{Builder, Lcg, Naming};
    use crate::record::v2;

    /// Session id of the sparse oracle recording.
    pub const SESSION_ID: &str = "sess-synthetic-sparse-oracle";

    /// Adapter name stamped on every record.
    pub const ADAPTER: &str = "synthetic-sparse-oracle-adapter";

    /// Origin timestamp, as RFC 3339. Distinct from the legible oracle's, so two
    /// dumps can never be confused for one another.
    pub const ORIGIN: &str = "2026-04-01T00:00:00Z";

    /// Period between baseline calls, in milliseconds.
    pub const BASELINE_PERIOD_MS: u64 = 30_000;
    /// End of the first baseline and start of the motif region.
    pub const FIRST_MOTIF_START_MS: u64 = 300_000;
    /// End of the motif region.
    pub const FIRST_MOTIF_END_MS: u64 = 540_000;
    /// Period between motif instances. 16 base samples at 500 ms: dyadic.
    pub const MOTIF_PERIOD_MS: u64 = 8_000;
    /// Records one motif instance contributes.
    pub const MOTIF_RECORDS_PER_INSTANCE: usize = 5;
    /// Start of the long regime block.
    pub const REGIME_START_MS: u64 = 780_000;
    /// End of the long regime block. 300 s wide: deliberately not a power of two
    /// in base samples.
    pub const REGIME_END_MS: u64 = 1_080_000;
    /// Period between calls inside the regime block.
    pub const REGIME_PERIOD_MS: u64 = 6_000;
    /// Start of the motif's recurrence.
    pub const SECOND_MOTIF_START_MS: u64 = REGIME_END_MS;
    /// End of the recurrence, and the session boundary.
    pub const SESSION_END_MS: u64 = 1_200_000;

    /// Tool name delivered by the baselines and by the motif's first call.
    pub const TOOL_READER: &str = "SparseSyntheticReader";
    /// Tool name delivered by the motif's second call, and nowhere else.
    pub const TOOL_SEARCHER: &str = "SparseSyntheticSearcher";
    /// Tool name delivered inside the regime block, and nowhere else.
    pub const TOOL_SHELL: &str = "SparseSyntheticShell";

    /// The sparse oracle recording, as complete v2 records in canonical order.
    pub fn records() -> Vec<v2::Record> {
        let mut builder = Builder::new(Naming {
            session_id: SESSION_ID,
            adapter: ADAPTER,
            prompt_id: "prompt-synthetic-sparse-0001",
            call_prefix: "toolu_synthetic_sparse_",
            origin: ORIGIN,
        });

        builder.session_started(0);
        baseline(&mut builder, 15_000, FIRST_MOTIF_START_MS);
        motif(
            &mut builder,
            FIRST_MOTIF_START_MS,
            FIRST_MOTIF_END_MS,
            false,
        );
        baseline(&mut builder, 555_000, REGIME_START_MS);
        regime(&mut builder);
        motif(&mut builder, SECOND_MOTIF_START_MS, SESSION_END_MS, true);
        builder.session_ended(SESSION_END_MS);

        builder.records
    }

    /// The sparse oracle recording, as the NDJSON bytes of a complete recording.
    ///
    /// One newline-terminated line per record, which is what
    /// `fixtures/synthetic-behavioral-oracle-sparse.ndjson` holds.
    pub fn ndjson() -> String {
        let mut out = String::new();
        for record in records() {
            // Serializing a record this module built cannot fail; the arm keeps
            // the function total without fabricating a line.
            if let Ok(line) = serde_json::to_string(&record) {
                out.push_str(&line);
                out.push('\n');
            }
        }
        out
    }

    /// One two-record call every [`BASELINE_PERIOD_MS`], landing in one bucket.
    fn baseline(builder: &mut Builder, from_ms: u64, until_ms: u64) {
        let mut at = from_ms;
        while at < until_ms {
            let id = builder.next_call_id();
            let input = serde_json::json!({ "path": "/synthetic/sparse/baseline.txt" });
            builder.tool_requested(at, TOOL_READER, &id, input.clone());
            builder.tool_succeeded(
                at + 120,
                TOOL_READER,
                &id,
                input,
                serde_json::json!({ "bytes": 32 }),
            );
            at += BASELINE_PERIOD_MS;
        }
    }

    /// A reported intent and two calls, repeated at [`MOTIF_PERIOD_MS`].
    ///
    /// `noisy` jitters the instance start and each record's offset, and makes one
    /// call fail. The jitter is a fixed generator seeded from the interval, so
    /// "noise" here is reproducible — the only kind an oracle can have.
    fn motif(builder: &mut Builder, from_ms: u64, until_ms: u64, noisy: bool) {
        let mut jitter = Lcg::new(0x5350_4152_5345_0001 ^ from_ms);
        let mut nominal = from_ms;
        let mut instance = 0u64;

        while nominal < until_ms {
            let at = if noisy {
                nominal + jitter.next_below(700)
            } else {
                nominal
            };
            let skew = |offset: u64, jitter: &mut Lcg| {
                if noisy {
                    at + offset + jitter.next_below(150)
                } else {
                    at + offset
                }
            };

            builder.reported_intent(
                at,
                &builder.peek_call_id(),
                "run the sparse synthetic motif over the synthetic tree",
            );

            for (index, (tool, request_offset, response_offset)) in
                [(TOOL_READER, 150, 400), (TOOL_SEARCHER, 700, 950)]
                    .into_iter()
                    .enumerate()
            {
                let id = builder.next_call_id();
                let input = serde_json::json!({
                    "target": "/synthetic/sparse/motif",
                    "step": index,
                });
                builder.tool_requested(skew(request_offset, &mut jitter), tool, &id, input.clone());

                // The recurrence's one structural deviation, matching the legible
                // oracle's: a single failing call rather than a perfect repeat.
                if noisy && instance == 3 && tool == TOOL_SEARCHER {
                    builder.tool_failed(
                        skew(response_offset, &mut jitter),
                        tool,
                        &id,
                        input,
                        "synthetic failure injected by the sparse oracle fixture",
                    );
                } else {
                    builder.tool_succeeded(
                        skew(response_offset, &mut jitter),
                        tool,
                        &id,
                        input,
                        serde_json::json!({ "matches": 2 }),
                    );
                }
            }

            instance += 1;
            nominal += MOTIF_PERIOD_MS;
        }
    }

    /// The long block: one call every [`REGIME_PERIOD_MS`] under a tool name that
    /// appears nowhere else, with one subagent pair inside it.
    fn regime(builder: &mut Builder) {
        let mut at = REGIME_START_MS;
        let mut subagent_started = false;
        let mut subagent_stopped = false;

        while at < REGIME_END_MS {
            if !subagent_started && at >= 840_000 {
                builder.subagent_started(at, "agent-synthetic-sparse-child");
                subagent_started = true;
            }
            if !subagent_stopped && at >= 1_020_000 {
                builder.subagent_stopped(at, "agent-synthetic-sparse-child");
                subagent_stopped = true;
            }

            let id = builder.next_call_id();
            let input = serde_json::json!({
                "command": "synthetic-sparse-build --target /synthetic/sparse/block",
            });
            builder.tool_requested(at, TOOL_SHELL, &id, input.clone());
            builder.tool_succeeded(
                at + 250,
                TOOL_SHELL,
                &id,
                input,
                serde_json::json!({ "exit_status": 0, "stdout": "synthetic block output" }),
            );

            at += REGIME_PERIOD_MS;
        }
    }
}
