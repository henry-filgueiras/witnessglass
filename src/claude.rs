//! Passive Claude Code command-hook adapter.
//!
//! This module turns exactly one Claude Code hook payload into zero or more
//! schema v2 emissions. It is passive by construction: it reads a payload,
//! translates it, and returns. It never decides anything, never rewrites
//! anything, and has no way to influence the session it is recording.
//!
//! # What passive means here, precisely
//!
//! The CLI wrapping this module prints nothing on stdout, so it cannot return a
//! permission decision, an updated tool input, an updated tool output, or
//! additional context — all of which Claude reads from a hook's stdout. It never
//! exits 2, which is the exit code Claude treats as a block for the hooks that
//! honor it. It never reads the transcript. It never executes or interpolates
//! any value out of the payload: every value is either compared against a fixed
//! set of names or stored as opaque JSON.
//!
//! # Fidelity, stated up front
//!
//! This adapter sees exactly what Claude Code's command hooks deliver, and the
//! evidence it produces is worth exactly that much:
//!
//! * A pre-tool payload is a **request**. Claude documents it as firing after
//!   the model constructs a tool request and before the call is processed. The
//!   request may then be modified, denied, deferred, or never executed. It is
//!   recorded as [`ToolRequested`](crate::record::v2::ToolRequested) and must
//!   never be read as proof that anything ran.
//! * A completion payload carries Claude's **effective** tool input and the
//!   tool-level response. It is not an account of what the tool did internally,
//!   and it is not a record of descendant processes.
//! * Validation rejections can fire neither the pre hook nor the failure hook,
//!   so a request that never reaches execution can leave no trace at all.
//! * Permission denial coverage depends on the documented event's actual
//!   behavior on this host and version, which is unmeasured until first contact.
//! * `@`-style file references may bypass `Read` tool hooks, so file content
//!   can enter a session without any tool event.
//!
//! Those are provisional statements taken from documentation. Nothing in this
//! module has been exercised against a live session yet; that is task:4.

use serde::Deserialize;

use crate::record::v2::{
    Context, Emission, Event, ReportedIntent, SessionEnded, SessionStarted, SubagentStarted,
    SubagentStopped, ToolDenied, ToolFailed, ToolRequested, ToolSucceeded,
};
use crate::record::{Channel, Provenance};

/// Adapter name recorded in every record this module produces.
pub const ADAPTER: &str = "claude-code";

/// File extension for a recording.
const RECORDING_EXTENSION: &str = "ndjson";

/// Longest session id accepted as a filename stem. Claude session ids are
/// UUIDs; this is generous and exists only to keep a pathological value from
/// reaching the filesystem.
const MAX_SESSION_ID_LEN: usize = 128;

/// Hook events this adapter supports.
///
/// Anything else is refused by name rather than guessed at. A hook Claude adds
/// later is not silently reinterpreted as one of these.
const SUPPORTED_HOOKS: &[&str] = &[
    "SessionStart",
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PermissionDenied",
    "SubagentStart",
    "SubagentStop",
    "SessionEnd",
];

/// Payload fields this adapter has seen, understands, and deliberately does not
/// record. Compile-time, and the second half of a complete statement: a field is
/// accounted for if it is either a [`HookPayload`] field or named here.
///
/// This list exists so that "we drop this on purpose" and "we have never heard
/// of this" are different facts. Without it, strict mode would reject every real
/// payload on `cwd` alone and be useless. Each entry needs a reason, and a field
/// whose reason has expired should be promoted into the struct rather than left
/// here.
const DELIBERATELY_UNRECORDED: &[&str] = &[
    // Absolute working directory. Privacy: it names the operator's filesystem on
    // every record (CLAUDE.md §5, dragon:2). Note that dragon:2 also found it
    // arriving *inside* error strings, where dropping the field cannot help.
    "cwd",
    // Absolute path to the session's full transcript, usually under $HOME. A
    // pointer to far more than a recording holds, and to material this project
    // has made no claims about.
    "transcript_path",
    // The permission mode in force. Not modelled, and dragon:1 argues it should
    // be: this session's entire denial result is explained by it, and the
    // recording cannot state it. Promoting it is a schema change and wants a
    // decision, not a quiet addition here.
    "permission_mode",
    // Reasoning-effort descriptor for the agent. Bears on how the agent thought,
    // not on what the machinery observed it do.
    "effort",
    // Documented on SessionStart; never observed on this host. Unmodelled rather
    // than assumed absent.
    "model",
    // Assistant prose on SubagentStop. This is the reported channel and would
    // need decision:2's treatment rather than a field slot, and it is a large
    // privacy surface for the value it adds.
    "last_assistant_message",
    // Why a subagent stopped. Unmodelled; nothing derives from it yet.
    "stop_reason",
];

/// What to do about payload fields that are in neither [`HookPayload`] nor
/// [`DELIBERATELY_UNRECORDED`].
///
/// Ignoring is the default and the only safe production posture: rejecting means
/// a recording stops the day Claude adds a field, which is a worse outcome than
/// a recording that misses one. Rejecting is a canary you run deliberately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnmodelledFields {
    /// Drop them, as this adapter always has.
    #[default]
    Ignore,
    /// Refuse the payload and name them.
    Reject,
}

/// Why a hook payload could not be translated.
///
/// Every variant means nothing was appended. The adapter either records a hook
/// completely or records none of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookError {
    /// The payload was not a single well-formed JSON object with the fields
    /// every hook carries.
    Malformed(String),
    /// A hook event this adapter does not support. Refused rather than guessed
    /// at: inventing a meaning for an unrecognized lifecycle point is how a
    /// recording acquires evidence nobody generated.
    UnsupportedHookEvent(String),
    /// A supported hook arrived without a field its translation requires.
    MissingField {
        /// Hook event name.
        hook_event_name: String,
        /// Field the translation needed.
        field: &'static str,
    },
    /// The `session_id` could not be used as a filename safely.
    UnsafeSessionId(String),
    /// Strict mode was asked for and the payload carried fields this adapter has
    /// no model for. Not a defect in the session — evidence that the wire has
    /// moved and this adapter has not.
    UnmodelledFields {
        /// Hook event name.
        hook_event_name: String,
        /// The unaccounted-for field names, sorted.
        fields: Vec<String>,
    },
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::Malformed(reason) => {
                write!(f, "malformed hook payload: {reason}")
            }
            HookError::UnsupportedHookEvent(name) => write!(
                f,
                "unsupported hook_event_name {name:?}; this adapter supports {}",
                SUPPORTED_HOOKS.join(", ")
            ),
            HookError::MissingField {
                hook_event_name,
                field,
            } => write!(
                f,
                "{hook_event_name} payload is missing required field {field:?}"
            ),
            HookError::UnsafeSessionId(reason) => {
                write!(f, "refusing to use session_id as a filename: {reason}")
            }
            HookError::UnmodelledFields {
                hook_event_name,
                fields,
            } => write!(
                f,
                "{hook_event_name} payload carried {} field(s) this adapter has no model for: {}. \
                 Strict mode refused it and appended nothing. Without --strict-json-validation \
                 these are dropped silently, which is how the delivered duration went unrecorded \
                 for two sprints. Either model them, or add them to DELIBERATELY_UNRECORDED with \
                 a reason",
                fields.len(),
                fields.join(", ")
            ),
        }
    }
}

impl std::error::Error for HookError {}

/// One Claude Code command-hook payload, as delivered on stdin.
///
/// Unknown fields are deliberately ignored. Claude adds fields to these payloads
/// over time, and a `deny_unknown_fields` here would mean a harmless addition
/// silently disabled recording for every session on the host. The strictness
/// lives where it belongs — on the record written out — rather than on the wire
/// format of somebody else's tool.
///
/// Everything is optional except the two fields every documented payload
/// carries, because which fields are present is exactly what varies between
/// hooks, and an absent field is evidence about coverage rather than an error.
#[derive(Debug, Clone, Deserialize)]
struct HookPayload {
    hook_event_name: String,
    session_id: String,

    // Common causal context.
    #[serde(default)]
    prompt_id: Option<String>,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    agent_type: Option<String>,

    // Session boundaries.
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    reason: Option<String>,

    // Tool lifecycle.
    #[serde(default)]
    tool_use_id: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    tool_input: Option<serde_json::Value>,
    #[serde(default)]
    tool_response: Option<serde_json::Value>,
    // `duration_ms` and `is_interrupt` are named for the wire, not for the
    // record. The adapter originally read `duration` and `interrupted`, the
    // names the hooks reference documents, and the observed payloads use
    // neither. Because unknown fields are ignored here — deliberately, see
    // above — both fields were dropped in silence, and a field-population
    // count over the resulting recording reported them as never supplied.
    // dragon:1 carried that as a coverage gap for two sprints. Renaming these
    // to the wire's own spelling keeps the mismatch visible to the next
    // reader; the record-side names are unchanged.
    #[serde(default)]
    duration_ms: Option<u64>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    is_interrupt: Option<bool>,

    // Subagent lifecycle. Recorded only when supplied; never synthesized.
    #[serde(default)]
    parent_agent_id: Option<String>,
    #[serde(default)]
    parent_agent_type: Option<String>,

    /// Everything above did not claim. Captured rather than discarded so the
    /// adapter can say *which* fields it is dropping instead of only that it
    /// drops some.
    ///
    /// `flatten` is what keeps this honest: the set of modelled fields is the
    /// struct itself, so adding a field above removes it from here automatically
    /// and no second list can fall out of step with the first. A hand-maintained
    /// roster of known keys would have needed updating in the same commit that
    /// introduced the `duration` bug, by the same person who introduced it.
    #[serde(flatten)]
    unmodelled: std::collections::BTreeMap<String, serde_json::Value>,
}

impl HookPayload {
    /// Field names present on the wire that neither the struct nor
    /// [`DELIBERATELY_UNRECORDED`] accounts for. Sorted, because `BTreeMap`.
    fn unaccounted_fields(&self) -> Vec<String> {
        self.unmodelled
            .keys()
            .filter(|key| !DELIBERATELY_UNRECORDED.contains(&key.as_str()))
            .cloned()
            .collect()
    }
}

/// The result of translating one hook payload.
#[derive(Debug, Clone, PartialEq)]
pub struct Translation {
    /// Session the payload belongs to, as delivered.
    pub session_id: String,
    /// Validated file name for this session's recording, relative to the
    /// recordings directory. Contains no separator and no `.` component.
    pub file_name: String,
    /// Emissions to append, in order. Usually one; two when a tool request
    /// carried an explicit agent-authored description.
    pub emissions: Vec<Emission>,
}

/// Translate exactly one Claude Code command-hook payload, dropping fields this
/// adapter has no model for.
///
/// Returns every emission the payload supports, in the order they should be
/// appended. Returns an error without producing any emission if the payload is
/// malformed, names an unsupported hook, is missing a field its translation
/// needs, or carries a `session_id` that cannot safely become a filename.
pub fn translate(payload_json: &str) -> Result<Translation, HookError> {
    translate_with(payload_json, UnmodelledFields::Ignore)
}

/// Translate one payload, choosing what happens to fields this adapter has no
/// model for.
///
/// [`UnmodelledFields::Reject`] is a drift canary, not a hardening measure. It
/// answers "has the wire moved since anybody looked", which is a question this
/// project learned to ask the expensive way: the delivered `duration_ms` was
/// discarded unread from the day the adapter was written, and every check that
/// went looking for it inspected the adapter's own output. A payload field the
/// adapter cannot name is the general shape of that failure.
///
/// It is deliberately not the default. Rejecting means no record is written, so
/// leaving it on turns a harmless upstream addition into a recording that stops
/// mid-session — the exact outcome [`HookPayload`]'s leniency exists to prevent.
/// Arm with it for one session, read what it says, and turn it off.
pub fn translate_with(
    payload_json: &str,
    unmodelled: UnmodelledFields,
) -> Result<Translation, HookError> {
    let payload: HookPayload = serde_json::from_str(payload_json.trim())
        .map_err(|err| HookError::Malformed(err.to_string()))?;

    if !SUPPORTED_HOOKS.contains(&payload.hook_event_name.as_str()) {
        return Err(HookError::UnsupportedHookEvent(payload.hook_event_name));
    }

    if unmodelled == UnmodelledFields::Reject {
        let fields = payload.unaccounted_fields();
        if !fields.is_empty() {
            return Err(HookError::UnmodelledFields {
                hook_event_name: payload.hook_event_name,
                fields,
            });
        }
    }

    let file_name = recording_file_name(&payload.session_id)?;
    let emissions = translate_payload(&payload)?;

    Ok(Translation {
        session_id: payload.session_id,
        file_name,
        emissions,
    })
}

/// Validate a session id and turn it into a recording file name.
///
/// The id is checked against a closed character set rather than escaped. A
/// Claude session id is a UUID, so anything outside `[A-Za-z0-9_-]` means the
/// adapter has met something it was not built for, and failing loudly is better
/// than inventing an encoding whose inverse nobody has defined. `.` is excluded
/// from the set entirely, which makes `.` and `..` unrepresentable and removes
/// the traversal question rather than answering it.
pub fn recording_file_name(session_id: &str) -> Result<String, HookError> {
    if session_id.is_empty() {
        return Err(HookError::UnsafeSessionId("it is empty".to_owned()));
    }
    if session_id.len() > MAX_SESSION_ID_LEN {
        return Err(HookError::UnsafeSessionId(format!(
            "it is {} bytes, longer than the {MAX_SESSION_ID_LEN}-byte limit",
            session_id.len()
        )));
    }
    if let Some(bad) = session_id
        .chars()
        .find(|c| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-'))
    {
        return Err(HookError::UnsafeSessionId(format!(
            "it contains {bad:?}, which is outside the permitted set [A-Za-z0-9_-]"
        )));
    }
    Ok(format!("{session_id}.{RECORDING_EXTENSION}"))
}

/// Provenance for an observed record from a given hook.
fn observed(hook_event_name: &str) -> Provenance {
    Provenance {
        channel: Channel::Observed,
        adapter: ADAPTER.to_owned(),
        mechanism: format!("command-hook:{hook_event_name}"),
    }
}

/// Causal context carried by tool and intent records.
///
/// Subagent lifecycle events deliberately do not use this: their `agent_id`
/// names the subagent being started or stopped, not the agent that emitted the
/// event, so it belongs in the event payload where it cannot be misread as the
/// emitter's identity.
fn context_of(payload: &HookPayload) -> Context {
    Context {
        prompt_id: payload.prompt_id.clone(),
        agent_id: payload.agent_id.clone(),
        agent_type: payload.agent_type.clone(),
    }
}

fn require<'a, T>(
    payload: &'a HookPayload,
    field: &'static str,
    value: &'a Option<T>,
) -> Result<&'a T, HookError> {
    value.as_ref().ok_or_else(|| HookError::MissingField {
        hook_event_name: payload.hook_event_name.clone(),
        field,
    })
}

fn translate_payload(payload: &HookPayload) -> Result<Vec<Emission>, HookError> {
    let hook = payload.hook_event_name.as_str();

    let emissions = match hook {
        "SessionStart" => vec![Emission {
            session_id: payload.session_id.clone(),
            context: context_of(payload),
            provenance: observed(hook),
            event: Event::SessionStarted(SessionStarted {
                source: payload.source.clone(),
            }),
        }],

        "SessionEnd" => vec![Emission {
            session_id: payload.session_id.clone(),
            context: context_of(payload),
            provenance: observed(hook),
            event: Event::SessionEnded(SessionEnded {
                reason: payload.reason.clone(),
            }),
        }],

        "PreToolUse" => {
            let tool_use_id = require(payload, "tool_use_id", &payload.tool_use_id)?;
            let tool_name = require(payload, "tool_name", &payload.tool_name)?;
            let tool_input = require(payload, "tool_input", &payload.tool_input)?;

            let mut emissions = vec![Emission {
                session_id: payload.session_id.clone(),
                context: context_of(payload),
                provenance: observed(hook),
                event: Event::ToolRequested(ToolRequested {
                    tool_use_id: tool_use_id.clone(),
                    tool_name: tool_name.clone(),
                    requested_input: tool_input.clone(),
                }),
            }];

            // An explicit agent-authored description inside the requested input
            // is a claim about intent, and it is the only thing in this payload
            // that is. It is lifted into its own reported record so that the
            // epistemic channel is visible in the record's own provenance rather
            // than buried inside a JSON blob classified as an observation. The
            // full requested input is still preserved above, unedited, as
            // source-delivered evidence; the description is duplicated, not
            // moved.
            //
            // Nothing else in the payload produces intent. A command, a path, a
            // tool name, a result, or a neighbouring record in time is not the
            // agent saying anything.
            if let Some(text) = explicit_description(tool_input) {
                emissions.push(Emission {
                    session_id: payload.session_id.clone(),
                    context: context_of(payload),
                    provenance: Provenance {
                        channel: Channel::Reported,
                        adapter: ADAPTER.to_owned(),
                        mechanism: format!("command-hook:{hook}#tool_input.description"),
                    },
                    event: Event::ReportedIntent(ReportedIntent {
                        text,
                        tool_use_id: Some(tool_use_id.clone()),
                    }),
                });
            }

            emissions
        }

        "PostToolUse" => {
            let tool_use_id = require(payload, "tool_use_id", &payload.tool_use_id)?;
            let tool_name = require(payload, "tool_name", &payload.tool_name)?;
            let tool_input = require(payload, "tool_input", &payload.tool_input)?;
            let tool_response = require(payload, "tool_response", &payload.tool_response)?;

            vec![Emission {
                session_id: payload.session_id.clone(),
                context: context_of(payload),
                provenance: observed(hook),
                event: Event::ToolSucceeded(ToolSucceeded {
                    tool_use_id: tool_use_id.clone(),
                    tool_name: tool_name.clone(),
                    effective_input: tool_input.clone(),
                    response: tool_response.clone(),
                    duration_ms: payload.duration_ms,
                }),
            }]
        }

        "PostToolUseFailure" => {
            let tool_use_id = require(payload, "tool_use_id", &payload.tool_use_id)?;
            let tool_name = require(payload, "tool_name", &payload.tool_name)?;
            let tool_input = require(payload, "tool_input", &payload.tool_input)?;
            let error = require(payload, "error", &payload.error)?;

            vec![Emission {
                session_id: payload.session_id.clone(),
                context: context_of(payload),
                provenance: observed(hook),
                event: Event::ToolFailed(ToolFailed {
                    tool_use_id: tool_use_id.clone(),
                    tool_name: tool_name.clone(),
                    effective_input: tool_input.clone(),
                    error: error.clone(),
                    interrupted: payload.is_interrupt,
                    duration_ms: payload.duration_ms,
                }),
            }]
        }

        "PermissionDenied" => {
            let tool_use_id = require(payload, "tool_use_id", &payload.tool_use_id)?;
            let tool_name = require(payload, "tool_name", &payload.tool_name)?;
            let tool_input = require(payload, "tool_input", &payload.tool_input)?;

            vec![Emission {
                session_id: payload.session_id.clone(),
                context: context_of(payload),
                provenance: observed(hook),
                event: Event::ToolDenied(ToolDenied {
                    tool_use_id: tool_use_id.clone(),
                    tool_name: tool_name.clone(),
                    requested_input: tool_input.clone(),
                }),
            }]
        }

        // The identifier in a subagent lifecycle payload names the child. It is
        // filed as the child, and no parent is manufactured for it: where Claude
        // supplies a parent identifier it is carried through as delivered, and
        // where it does not, the field stays absent. Adjacency in the recording
        // is not parentage.
        "SubagentStart" => {
            let agent_id = require(payload, "agent_id", &payload.agent_id)?;
            vec![Emission {
                session_id: payload.session_id.clone(),
                context: Context {
                    prompt_id: payload.prompt_id.clone(),
                    agent_id: None,
                    agent_type: None,
                },
                provenance: observed(hook),
                event: Event::SubagentStarted(SubagentStarted {
                    agent_id: agent_id.clone(),
                    agent_type: payload.agent_type.clone(),
                    parent_agent_id: payload.parent_agent_id.clone(),
                    parent_agent_type: payload.parent_agent_type.clone(),
                }),
            }]
        }

        "SubagentStop" => {
            let agent_id = require(payload, "agent_id", &payload.agent_id)?;
            vec![Emission {
                session_id: payload.session_id.clone(),
                context: Context {
                    prompt_id: payload.prompt_id.clone(),
                    agent_id: None,
                    agent_type: None,
                },
                provenance: observed(hook),
                event: Event::SubagentStopped(SubagentStopped {
                    agent_id: agent_id.clone(),
                    agent_type: payload.agent_type.clone(),
                    parent_agent_id: payload.parent_agent_id.clone(),
                    parent_agent_type: payload.parent_agent_type.clone(),
                }),
            }]
        }

        // Unreachable: the caller checked the name against SUPPORTED_HOOKS.
        other => return Err(HookError::UnsupportedHookEvent(other.to_owned())),
    };

    Ok(emissions)
}

/// An explicit, agent-authored description inside a requested tool input.
///
/// The only thing recognized is a top-level `description` string that is not
/// blank. That is a field the agent filled in with prose about its own
/// intentions — Claude's `Bash` tool is the obvious case. Nothing is derived
/// from any other field: a command is not a description of why it was run.
fn explicit_description(tool_input: &serde_json::Value) -> Option<String> {
    let text = tool_input.get("description")?.as_str()?;
    if text.trim().is_empty() {
        None
    } else {
        Some(text.to_owned())
    }
}
