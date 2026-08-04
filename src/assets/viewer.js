// The WitnessGlass evidence workbench.
//
// This file renders what Rust already derived. It does not parse NDJSON, does
// not redefine lifecycle semantics, and does not invent a correlation the
// projection did not license. Where it needs to know what an event means, it
// reads a field the projection supplied.
//
// Counting is the place that rule is easiest to overstate, so it is stated
// precisely. This file chooses no membership, grouping, or rollup with meaning
// for the recording: which records support a claim, and what a number about
// them signifies, is decided in Rust and arrives with receipts and an examined
// scope. What it does compute is the size of a receipt set Rust handed it, and
// transient interface numbers such as how many rows survived a filter. Counting
// a set somebody else defined is not the same act as deciding what belongs in
// one.
//
// Two rules hold everywhere in this file:
//
// 1. No recording text ever becomes markup. Every value from the projection
//    reaches the page through `textContent` or an attribute set with
//    `setAttribute`. Not one HTML-parsing sink is used anywhere in this file —
//    no markup-assigning property, no adjacent-markup insertion, no document
//    writing, no range fragment parser, no Markdown, and no linkification. A
//    recording contains commands and file contents chosen by nobody
//    trustworthy. `tests/workbench.rs` asserts the absence of each sink by
//    name, which is why none of their names appears in this comment.
//
// 2. Nothing is persisted. No storage, no cookie, no worker, no cache. The
//    capability lives in a module-scoped constant read from the URL and is
//    never written anywhere. Perspective, selection, filters, and search live
//    in one module-scoped object and die with the tab.
//
// Three perspectives, one dominant workflow. Events answers "what happened and
// where should I look"; Coverage answers "what was and was not captured";
// Provenance answers "where did this recording come from". The first-use pass
// in task:11 found the previous single-column layout put twelve peer panels in
// front of the investigative loop, so the loop comes first now and everything
// else is one keystroke away with its receipts intact.

const CAPABILITY = new URLSearchParams(location.search).get("c") || "";

// Receipt lists longer than this collapse behind a disclosure. Progressive
// disclosure, not receipt deletion: every sequence stays one click away.
const INLINE_RECEIPT_LIMIT = 8;

// ---------------------------------------------------------------------------
// DOM helpers. `text` is the only way content gets in.
// ---------------------------------------------------------------------------

function el(tag, options = {}, children = []) {
  const node = document.createElement(tag);
  if (options.class) node.className = options.class;
  if (options.text !== undefined && options.text !== null) node.textContent = String(options.text);
  if (options.attrs) {
    for (const [name, value] of Object.entries(options.attrs)) {
      if (value !== null && value !== undefined) node.setAttribute(name, String(value));
    }
  }
  // Positions go through the CSSOM, not through a `style` attribute. The
  // content security policy is `style-src 'self'`, which refuses inline style
  // attributes and would silently stack every mark at zero; widening it to
  // 'unsafe-inline' to move a dot would be a poor trade.
  if (options.style) {
    for (const [property, value] of Object.entries(options.style)) {
      node.style.setProperty(property, value);
    }
  }
  if (options.on) {
    for (const [event, handler] of Object.entries(options.on)) node.addEventListener(event, handler);
  }
  for (const child of children) {
    if (child) node.appendChild(child);
  }
  return node;
}

function clear(node) {
  while (node.firstChild) node.removeChild(node.firstChild);
}

// ---------------------------------------------------------------------------
// Reading the projection's tagged shapes.
// ---------------------------------------------------------------------------

/** `{v2: "tool_requested"}` -> `{schema: "v2", name: "tool_requested"}`. */
function readKind(kind) {
  const schema = Object.keys(kind)[0];
  return { schema, name: kind[schema], label: `${schema}:${kind[schema]}` };
}

/** `{v2_tool_use_id: "toolu_x"}` -> `{scheme, id, key}`, or null. */
function readCorrelation(correlation) {
  if (!correlation) return null;
  const scheme = Object.keys(correlation)[0];
  const id = correlation[scheme];
  // A v1 tool_call_id and a v2 tool_use_id are different mechanisms. The key
  // carries the scheme so two ids spelled the same never collapse into one.
  return { scheme, id, key: `${scheme} ${id}` };
}

/** The three states of current-agent attribution, kept apart. */
function readAgent(attribution) {
  if (attribution === "not_representable") {
    return { state: "not_representable", label: "no causal context in this schema", id: null };
  }
  if (attribution && attribution.supplied) {
    const { agent_id, agent_type } = attribution.supplied;
    return {
      state: "supplied",
      id: agent_id,
      type: agent_type,
      label: agent_type ? `${agent_id} (${agent_type})` : agent_id,
    };
  }
  const type = attribution && attribution.not_supplied ? attribution.not_supplied.agent_type : null;
  return {
    state: "not_supplied",
    id: null,
    type,
    label: "agent identity not supplied",
  };
}

function scopeLabel(scope) {
  if (scope && scope.complete_recording) {
    return `examined: all ${scope.complete_recording.records} record(s) of a complete recording`;
  }
  if (scope && scope.valid_prefix) {
    const p = scope.valid_prefix;
    return `examined: ${p.records} record(s) — the valid prefix of a recording that stops mid-record`;
  }
  return "examined: unknown scope";
}

function isTruncated(scope) {
  return Boolean(scope && scope.valid_prefix);
}

/** Anomalies, phrased as the sprint requires: absence as absence. */
const ANOMALY_TEXT = {
  missing_session_start: "no session_started record observed",
  missing_session_end: "no session_ended record observed",
  duplicate_session_start: "more than one session_started record",
  duplicate_session_end: "more than one session_ended record",
  duplicate_openings: "more than one request or claimed start for one correlation id",
  duplicate_outcomes: "more than one outcome record for one correlation id",
  conflicting_outcomes: "outcome records disagree about what became of this call",
  opening_without_outcome: "outcome not observed",
  outcome_without_opening: "no request or claimed start observed",
  reported_intent_without_observed_evidence: "a claim citing an id no observed record carries",
  divergent_tool_names: "records deliver different tool names; none is canonical",
  subagent_stop_without_start: "stop without observed start",
  subagent_start_without_stop: "start without observed stop",
  divergent_agent_types: "one agent id delivered with more than one type",
};

function readAnomaly(anomaly) {
  const kind = anomaly.kind;
  if (typeof kind === "string") {
    return { name: kind, text: ANOMALY_TEXT[kind] || kind, subject: null };
  }
  const name = Object.keys(kind)[0];
  const body = kind[name];
  let subject = null;
  if (body && body.id) {
    const correlation = readCorrelation(body.id);
    subject = correlation ? correlation.id : null;
  } else if (body && body.agent_id !== undefined) {
    subject = body.agent_id;
  }
  return { name, text: ANOMALY_TEXT[name] || name, subject };
}

const SHAPE_TEXT = {
  reported_intent_only: "claim only — no observed record carries this id",
  opening_without_outcome: "outcome not observed",
  outcome_without_opening: "request or claimed start not observed",
  paired_lifecycle: "one request and one outcome, correlated",
  ambiguous: "ambiguous — nothing was paired",
};

const COVERAGE_TEXT = {
  v2_duration_ms: "duration_ms, on completion and failure records",
  v2_interrupted: "interrupted, on failure records",
  v2_supplied_parent_agent: "parent identity, on subagent boundary records",
  v2_prompt_id: "prompt_id, on every record",
};

/**
 * The tool-lifecycle kinds of each schema, in the order they are worth reading.
 *
 * This is not a rollup. Each name is a kind Rust already counted with receipts
 * and an examined scope; the summary renders those counts individually and
 * never adds them together into an invented "outcomes" total.
 */
const LIFECYCLE_KINDS = {
  v1: ["observed_tool_started", "observed_tool_finished"],
  v2: ["tool_requested", "tool_succeeded", "tool_failed", "tool_denied"],
};

const CHANNEL_GLYPH = { reported: "◇", observed: "●", recorder: "▣" };

/** Channel marker: a glyph *and* a word, so colour is never the only cue. */
function channelTag(channel) {
  return el("span", { class: `tag tag-${channel}` }, [
    el("span", { class: "glyph", text: CHANNEL_GLYPH[channel] || "○", attrs: { "aria-hidden": "true" } }),
    el("span", { text: channel }),
  ]);
}

function derivedTag(text) {
  return el("span", { class: "tag tag-derived" }, [
    el("span", { class: "glyph", text: "⟐", attrs: { "aria-hidden": "true" } }),
    el("span", { text: text || "derived" }),
  ]);
}

// ---------------------------------------------------------------------------
// State. In memory only; it dies with the tab.
// ---------------------------------------------------------------------------

const state = {
  projection: null,
  index: new Map(), // sequence -> ledger entry
  anomaliesBySeq: new Map(),
  groupsByKey: new Map(),
  perspective: "events",
  selected: null,
  axis: "sequence",
  filters: {
    channel: new Set(),
    kind: new Set(),
    tool: new Set(),
    mechanism: new Set(),
    agent: new Set(),
    anomalous: false,
  },
  search: "",
  searchPayloads: false,
};

const FILTER_LABELS = {
  channel: "channel",
  kind: "kind",
  tool: "tool",
  mechanism: "mechanism",
  agent: "agent",
};

const UNATTRIBUTED = " unattributed";

function buildIndexes() {
  const p = state.projection;
  for (const entry of p.ledger) state.index.set(entry.sequence, entry);
  for (const anomaly of p.anomalies) {
    for (const sequence of anomaly.receipts) {
      if (!state.anomaliesBySeq.has(sequence)) state.anomaliesBySeq.set(sequence, []);
      state.anomaliesBySeq.get(sequence).push(anomaly);
    }
  }
  for (const group of p.tool_groups) {
    state.groupsByKey.set(readCorrelation(group.id).key, group);
  }
}

function entryAnomalies(sequence) {
  return state.anomaliesBySeq.get(sequence) || [];
}

function agentKeyOf(entry) {
  const agent = readAgent(entry.current_agent);
  return agent.state === "supplied" ? agent.id : UNATTRIBUTED;
}

function activeFilterCount() {
  const f = state.filters;
  return (
    f.channel.size + f.kind.size + f.tool.size + f.mechanism.size + f.agent.size +
    (f.anomalous ? 1 : 0)
  );
}

// ---------------------------------------------------------------------------
// Receipts: the link from a derived claim back to the records supporting it.
// ---------------------------------------------------------------------------

function receiptButton(sequence) {
  return el("button", {
    class: "receipt",
    text: `#${sequence}`,
    attrs: { type: "button", "aria-label": `Select record ${sequence}` },
    on: { click: () => select(sequence, { reveal: true }) },
  });
}

/**
 * Receipts for a derived claim.
 *
 * A short list stays inline. A long one collapses behind a disclosure naming
 * how many records support the claim, and builds its buttons the first time it
 * is opened — a recording with 82 correlated pairs otherwise renders several
 * hundred buttons nobody asked for. Collapsed is not deleted: every sequence is
 * one click away, and the count is always visible.
 */
function receiptList(sequences, options = {}) {
  const wrap = el("span", { class: "receipts" });
  if (!sequences.length) {
    wrap.appendChild(el("span", { class: "muted", text: options.emptyText || "no supporting record" }));
    return wrap;
  }
  if (sequences.length <= (options.limit || INLINE_RECEIPT_LIMIT)) {
    for (const sequence of sequences) wrap.appendChild(receiptButton(sequence));
    return wrap;
  }
  const holder = el("span", { class: "receipt-holder" });
  const details = el("details", { class: "receipt-set" }, [
    el("summary", { text: `${sequences.length} supporting records` }),
    holder,
  ]);
  let built = false;
  details.addEventListener("toggle", () => {
    if (!details.open || built) return;
    built = true;
    for (const sequence of sequences) holder.appendChild(receiptButton(sequence));
  });
  wrap.appendChild(details);
  return wrap;
}

/** A count, its receipts, and the scope that makes a zero readable. */
function countRow(label, recordCount, zeroPhrase) {
  const row = el("div", { class: "count-row" }, [
    el("span", { class: "count-label", text: label }),
    el("span", { class: "count-value", text: String(recordCount.records.length) }),
  ]);
  if (!recordCount.records.length) {
    row.appendChild(el("span", { class: "count-absent", text: zeroPhrase || "no matching record observed" }));
    row.appendChild(el("span", { class: "count-scope", text: scopeLabel(recordCount.scope) }));
  } else {
    row.appendChild(receiptList(recordCount.records));
  }
  return row;
}

function block(title, children, options = {}) {
  const head = el("h3", { text: title });
  if (options.derived) head.appendChild(derivedTag());
  return el("section", { class: `block${options.class ? " " + options.class : ""}` }, [head, ...children]);
}

// ---------------------------------------------------------------------------
// Events — the compact summary
// ---------------------------------------------------------------------------

function kindTally(name) {
  return state.projection.aggregates.by_event_kind.find((tally) => readKind(tally.value).name === name);
}

function statCell(label, value, options = {}) {
  const cell = el("div", { class: `stat${options.class ? " " + options.class : ""}` }, [
    el("span", { class: "stat-value", text: value }),
    el("span", { class: "stat-label", text: label }),
  ]);
  if (options.note) cell.appendChild(el("span", { class: "stat-note", text: options.note }));
  return cell;
}

function renderSummary() {
  const p = state.projection;
  const host = document.getElementById("summary");
  clear(host);

  const truncated = isTruncated(p.scope);
  host.appendChild(
    statCell(
      "completeness",
      truncated ? "ends mid-record" : "complete",
      {
        class: truncated ? "stat-alarm" : "",
        note: truncated
          ? `${p.scope.valid_prefix.fragment_bytes}-byte fragment at byte ${p.scope.valid_prefix.fragment_byte_offset}, not replayed; every absence below is scoped to this valid prefix`
          : "the final record was newline-terminated",
      },
    ),
  );
  host.appendChild(statCell("records", String(p.records.length), { note: `schema ${p.schema_version === null ? "not established" : "v" + p.schema_version}` }));

  // Each lifecycle kind, counted by Rust, phrased individually. No total.
  const schema = p.schema_version === 1 ? "v1" : p.schema_version === 2 ? "v2" : null;
  for (const name of LIFECYCLE_KINDS[schema] || []) {
    const tally = kindTally(name);
    if (!tally) continue;
    const count = tally.records.records.length;
    host.appendChild(
      statCell(name, String(count), {
        class: count === 0 ? "stat-absent" : "",
        note: count === 0 ? `no ${name} record observed — ${scopeLabel(tally.records.scope)}` : null,
      }),
    );
  }

  const anomalies = el("button", {
    class: `stat stat-action${p.anomalies.length ? " stat-alarm" : ""}`,
    attrs: { type: "button" },
    on: { click: () => showPerspective("coverage", { focus: true }) },
  }, [
    el("span", { class: "stat-value", text: String(p.anomalies.length) }),
    el("span", { class: "stat-label", text: "anomalies" }),
    el("span", { class: "stat-note", text: "open Coverage" }),
  ]);
  host.appendChild(anomalies);
}

// ---------------------------------------------------------------------------
// Events — the event map
// ---------------------------------------------------------------------------

function renderMap() {
  const p = state.projection;
  const body = document.getElementById("map-body");
  const note = document.getElementById("map-note");
  clear(body);

  note.textContent = state.axis === "sequence"
    ? "One mark per record, positioned by append sequence. Point events only: no execution bar is drawn, because no execution interval was observed."
    : "DERIVED VIEW: marks positioned by recorder wall-clock time, which is descriptive metadata. Spacing here is not execution duration, and this view never reorders the ledger.";

  const visible = new Set(filteredEntries().map((entry) => entry.sequence));
  const lanes = new Map();
  for (const entry of p.ledger) {
    const key = agentKeyOf(entry);
    if (!lanes.has(key)) lanes.set(key, []);
    lanes.get(key).push(entry);
  }

  const maxSequence = p.records.length || 1;
  let earliest = 0;
  let span = 1;
  if (p.timestamps) {
    earliest = Date.parse(p.timestamps.earliest.recorded_at);
    span = Math.max(1, Date.parse(p.timestamps.latest.recorded_at) - earliest);
  }

  for (const [key, laneEntries] of lanes) {
    const unattributed = key === UNATTRIBUTED;
    const lane = el("div", { class: "lane" }, [
      el("div", {
        class: `lane-label${unattributed ? " lane-unattributed" : ""}`,
        text: unattributed ? "identity not supplied" : key,
        attrs: {
          title: unattributed
            ? "No agent identity was supplied on these records. This is not a root agent."
            : `context.agent_id ${key}`,
        },
      }),
      el("span", { class: "lane-count muted", text: `${laneEntries.length}` }),
    ]);
    const track = el("div", { class: "lane-track" });
    for (const entry of laneEntries) {
      const kind = readKind(entry.kind);
      const anomalies = entryAnomalies(entry.sequence);
      const position = state.axis === "sequence"
        ? ((entry.sequence - 1) / Math.max(1, maxSequence - 1)) * 100
        : ((Date.parse(entry.recorded_at) - earliest) / span) * 100;
      const mark = el("button", {
        class: [
          "mark",
          `mark-${entry.channel}`,
          anomalies.length ? "mark-anomalous" : "",
          visible.has(entry.sequence) ? "" : "mark-filtered",
        ].filter(Boolean).join(" "),
        style: { left: `${position.toFixed(3)}%` },
        attrs: {
          type: "button",
          tabindex: "-1",
          "data-sequence": String(entry.sequence),
          "aria-label": `Record ${entry.sequence}, ${entry.channel}, ${kind.name}${anomalies.length ? ", anomalous" : ""}`,
          title: `#${entry.sequence} ${entry.channel} ${kind.name}${anomalies.length ? " — " + anomalies.map((a) => readAnomaly(a).text).join("; ") : ""}`,
        },
        on: {
          click: () => select(entry.sequence),
          keydown: (event) => onMarkKey(event, entry.sequence),
        },
      }, [el("span", { class: "mark-dot", attrs: { "aria-hidden": "true" } })]);
      track.appendChild(mark);
    }
    lane.appendChild(track);
    body.appendChild(lane);
  }

  body.appendChild(el("div", { class: "legend" }, [
    el("span", { class: "legend-item" }, [el("span", { class: "mark-sample mark-reported" }), el("span", { text: "◇ reported — a claim" })]),
    el("span", { class: "legend-item" }, [el("span", { class: "mark-sample mark-observed" }), el("span", { text: "● observed — witnessed by a capture point" })]),
    el("span", { class: "legend-item" }, [el("span", { class: "mark-sample mark-recorder" }), el("span", { text: "▣ recorder — asserted by the recorder" })]),
    el("span", { class: "legend-item" }, [el("span", { class: "mark-sample mark-anomalous mark-observed" }), el("span", { text: "✱ ringed — cited by an anomaly" })]),
    el("span", { class: "muted", text: "faded marks are filtered out of the ledger; arrow keys move between marks" }),
  ]));

  updateMarkTabStop();
}

/** Roving tabindex: one tab stop for the whole map, arrows move within it. */
function orderedMarks() {
  return [...document.querySelectorAll(".mark")].sort(
    (a, b) => Number(a.getAttribute("data-sequence")) - Number(b.getAttribute("data-sequence")),
  );
}

function updateMarkTabStop() {
  const marks = orderedMarks();
  if (!marks.length) return;
  const wanted = String(state.selected);
  const active = marks.find((mark) => mark.getAttribute("data-sequence") === wanted) || marks[0];
  for (const mark of marks) mark.setAttribute("tabindex", mark === active ? "0" : "-1");
}

function onMarkKey(event, sequence) {
  const keys = ["ArrowRight", "ArrowLeft", "Home", "End"];
  if (!keys.includes(event.key)) return;
  event.preventDefault();
  const marks = orderedMarks();
  const at = marks.findIndex((mark) => mark.getAttribute("data-sequence") === String(sequence));
  let next;
  if (event.key === "Home") next = marks[0];
  else if (event.key === "End") next = marks[marks.length - 1];
  else next = marks[at + (event.key === "ArrowRight" ? 1 : -1)];
  if (!next) return;
  select(Number(next.getAttribute("data-sequence")), { keepFocus: true });
  next.setAttribute("tabindex", "0");
  next.focus();
}

// ---------------------------------------------------------------------------
// Events — filters and search
// ---------------------------------------------------------------------------

function distinct(getter) {
  const values = new Set();
  for (const entry of state.projection.ledger) {
    const value = getter(entry);
    if (value !== null && value !== undefined) values.add(value);
  }
  return [...values];
}

function filterGroup(title, name, values, render) {
  const boxes = values.map((value) => {
    const input = el("input", {
      attrs: { type: "checkbox", value: String(value), checked: state.filters[name].has(value) ? "" : null },
      on: {
        change: (event) => {
          if (event.target.checked) state.filters[name].add(value);
          else state.filters[name].delete(value);
          onFiltersChanged();
        },
      },
    });
    return el("label", { class: "filter-option" }, [input, render ? render(value) : el("span", { text: String(value) })]);
  });
  return el("fieldset", { class: "filter-group" }, [el("legend", { text: title }), ...boxes]);
}

function renderFilters() {
  const host = document.getElementById("filters");
  clear(host);
  host.appendChild(filterGroup("Channel", "channel", distinct((e) => e.channel), (v) => channelTag(v)));
  host.appendChild(filterGroup("Kind", "kind", distinct((e) => readKind(e.kind).name)));
  const tools = distinct((e) => e.tool_name);
  if (tools.length) host.appendChild(filterGroup("Tool", "tool", tools));
  host.appendChild(filterGroup("Mechanism", "mechanism", distinct((e) => e.mechanism)));
  host.appendChild(filterGroup("Supplied agent", "agent", distinct(agentKeyOf), (value) =>
    el("span", { text: value === UNATTRIBUTED ? "identity not supplied" : value })));
}

function chip(label, onRemove) {
  return el("span", { class: "active-chip" }, [
    el("span", { text: label }),
    el("button", {
      class: "chip-remove",
      text: "×",
      attrs: { type: "button", "aria-label": `Remove filter ${label}` },
      on: { click: onRemove },
    }),
  ]);
}

function renderActiveFilters() {
  const host = document.getElementById("active-filters");
  const badge = document.getElementById("filter-badge");
  clear(host);

  const count = activeFilterCount();
  badge.hidden = count === 0;
  badge.textContent = String(count);

  if (!count) {
    host.hidden = true;
    return;
  }
  host.hidden = false;

  for (const [name, label] of Object.entries(FILTER_LABELS)) {
    for (const value of state.filters[name]) {
      const shown = value === UNATTRIBUTED ? "identity not supplied" : value;
      host.appendChild(chip(`${label}: ${shown}`, () => {
        state.filters[name].delete(value);
        renderFilters();
        onFiltersChanged();
      }));
    }
  }
  if (state.filters.anomalous) {
    host.appendChild(chip("anomalous only", () => {
      state.filters.anomalous = false;
      document.getElementById("filter-anomalous").checked = false;
      onFiltersChanged();
    }));
  }
  host.appendChild(el("button", {
    class: "chip-clear",
    text: "clear all",
    attrs: { type: "button" },
    on: {
      click: () => {
        for (const name of Object.keys(FILTER_LABELS)) state.filters[name].clear();
        state.filters.anomalous = false;
        document.getElementById("filter-anomalous").checked = false;
        renderFilters();
        onFiltersChanged();
      },
    },
  }));
}

function onFiltersChanged() {
  renderActiveFilters();
  renderLedger();
  renderMap();
}

function matchesSearch(entry) {
  if (!state.search) return true;
  const needle = state.search.toLowerCase();
  const kind = readKind(entry.kind);
  const correlation = readCorrelation(entry.correlation);
  const agent = readAgent(entry.current_agent);
  const metadata = [
    String(entry.sequence),
    entry.channel,
    entry.adapter,
    entry.mechanism,
    kind.name,
    entry.tool_name,
    correlation ? correlation.id : "",
    agent.id || "",
    entry.subject_agent ? entry.subject_agent.agent_id : "",
    entry.prompt_id || "",
  ].join(" ").toLowerCase();
  if (metadata.includes(needle)) return true;
  if (!state.searchPayloads) return false;
  // Opt-in only: this reads commands, file contents, and tool output.
  return JSON.stringify(entry.record).toLowerCase().includes(needle);
}

function filteredEntries() {
  const f = state.filters;
  return state.projection.ledger.filter((entry) => {
    if (f.channel.size && !f.channel.has(entry.channel)) return false;
    if (f.kind.size && !f.kind.has(readKind(entry.kind).name)) return false;
    if (f.tool.size && !(entry.tool_name && f.tool.has(entry.tool_name))) return false;
    if (f.mechanism.size && !f.mechanism.has(entry.mechanism)) return false;
    if (f.agent.size && !f.agent.has(agentKeyOf(entry))) return false;
    if (f.anomalous && !entryAnomalies(entry.sequence).length) return false;
    return matchesSearch(entry);
  });
}

// ---------------------------------------------------------------------------
// Events — the ledger
// ---------------------------------------------------------------------------

function renderLedger() {
  const body = document.getElementById("ledger-body");
  const count = document.getElementById("ledger-count");
  clear(body);

  const entries = filteredEntries();
  const total = state.projection.ledger.length;
  count.textContent = entries.length === total
    ? `${total} record(s), canonical append order`
    : `${entries.length} of ${total} shown — filtered; canonical append order is unchanged`;

  const table = el("table", { class: "ledger-table" }, [
    el("thead", {}, [el("tr", {}, [
      el("th", { text: "#", attrs: { scope: "col" } }),
      el("th", { text: "Channel", attrs: { scope: "col" } }),
      el("th", { text: "Kind", attrs: { scope: "col" } }),
      el("th", { text: "Tool", attrs: { scope: "col" } }),
      el("th", { text: "Agent", attrs: { scope: "col" } }),
      el("th", { text: "Anomaly", attrs: { scope: "col" } }),
    ])]),
  ]);

  const tbody = el("tbody");
  for (const entry of entries) {
    const kind = readKind(entry.kind);
    const agent = readAgent(entry.current_agent);
    const anomalies = entryAnomalies(entry.sequence);

    const anomalyCell = el("td", { class: "cell-anomaly" });
    if (anomalies.length === 1) {
      anomalyCell.appendChild(el("span", { class: "flag", text: readAnomaly(anomalies[0]).text }));
    } else if (anomalies.length > 1) {
      anomalyCell.appendChild(el("span", { class: "flag", text: `${anomalies.length} anomalies` }));
    }

    const row = el("tr", {
      class: [
        `row-${entry.channel}`,
        anomalies.length ? "row-anomalous" : "",
        state.selected === entry.sequence ? "row-selected" : "",
      ].filter(Boolean).join(" "),
      attrs: {
        tabindex: "0",
        "data-sequence": String(entry.sequence),
        "aria-selected": String(state.selected === entry.sequence),
      },
      on: {
        click: () => select(entry.sequence),
        keydown: (event) => onRowKey(event, entry.sequence),
      },
    }, [
      el("td", { class: "cell-seq mono", text: String(entry.sequence) }),
      el("td", {}, [channelTag(entry.channel)]),
      el("td", { class: "mono", text: kind.name }),
      el("td", { class: "mono", text: entry.tool_name || "—" }),
      agent.state === "supplied"
        ? el("td", { class: "mono small", text: agent.id })
        : el("td", { class: "small muted", text: agent.label }),
      anomalyCell,
    ]);
    tbody.appendChild(row);
  }
  table.appendChild(tbody);
  body.appendChild(table);

  if (!entries.length) {
    body.appendChild(el("p", { class: "note", text: "No record matches these filters. That is a statement about the filters, not about the recording." }));
  }
}

function onRowKey(event, sequence) {
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    select(sequence);
    return;
  }
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  event.preventDefault();
  const rows = [...document.querySelectorAll("#ledger-body tbody tr")];
  const at = rows.findIndex((row) => row.getAttribute("data-sequence") === String(sequence));
  const next = rows[at + (event.key === "ArrowDown" ? 1 : -1)];
  if (!next) return;
  next.focus();
  next.scrollIntoView({ block: "nearest" });
  select(Number(next.getAttribute("data-sequence")), { keepFocus: true });
}

// ---------------------------------------------------------------------------
// Events — the evidence inspector
// ---------------------------------------------------------------------------

/** Render a JSON value as structure and text. Never as markup. */
function renderJson(value, depth = 0) {
  if (value === null) return el("span", { class: "json-null", text: "null" });
  if (typeof value === "boolean") return el("span", { class: "json-bool", text: String(value) });
  if (typeof value === "number") return el("span", { class: "json-number", text: String(value) });
  if (typeof value === "string") return el("span", { class: "json-string", text: value });
  if (Array.isArray(value)) {
    if (!value.length) return el("span", { class: "muted", text: "[]" });
    return el("ul", { class: "json-list" }, value.map((item) => el("li", {}, [renderJson(item, depth + 1)])));
  }
  const rows = [];
  for (const [key, inner] of Object.entries(value)) {
    rows.push(el("div", { class: "json-row" }, [
      el("span", { class: "json-key", text: key }),
      renderJson(inner, depth + 1),
    ]));
  }
  return el("div", { class: "json-object" }, rows);
}

function collapsedPayload(title, value) {
  const details = el("details", { class: "payload" });
  details.appendChild(el("summary", { text: `${title} — reveal (sensitive)` }));
  let built = false;
  details.addEventListener("toggle", () => {
    if (!details.open || built) return;
    built = true;
    details.appendChild(renderJson(value));
  });
  return details;
}

function evidenceCard(entry, roleText) {
  const kind = readKind(entry.kind);
  const card = el("div", { class: `evidence evidence-${entry.channel}` }, [
    el("div", { class: "evidence-head" }, [
      receiptButton(entry.sequence),
      channelTag(entry.channel),
      el("span", { class: "mono strong", text: kind.name }),
      roleText ? el("span", { class: "muted", text: roleText }) : null,
    ]),
  ]);
  if (entry.facets.reported_text !== null && entry.facets.reported_text !== undefined) {
    card.appendChild(el("blockquote", { class: "claim", text: entry.facets.reported_text }));
    card.appendChild(el("p", { class: "note", text: "A claim the agent made about itself. It is not evidence that anything ran." }));
  }
  if (entry.tool_name) {
    card.appendChild(el("p", { class: "small" }, [
      el("span", { class: "muted", text: "tool name as delivered: " }),
      el("span", { class: "mono", text: entry.tool_name }),
    ]));
  }
  return card;
}

const OBSERVED_LABELS = {
  started: "claimed start (v1 claims a witnessed beginning)",
  finished_succeeded: "outcome: succeeded",
  finished_failed: "outcome: failed",
  requested: "request — not proof of execution",
  succeeded: "outcome: executed successfully",
  failed: "outcome: executed and failed",
  denied: "outcome: denied, did not execute",
};

function renderInspector() {
  const body = document.getElementById("inspector-body");
  clear(body);

  if (state.selected === null) {
    body.appendChild(el("p", { class: "note", text: "Select a record in the map or the ledger. Its envelope, provenance, context, and payload appear here, alongside — never merged with — whatever else shares its correlation id." }));
    return;
  }

  const entry = state.index.get(state.selected);
  if (!entry) return;
  const kind = readKind(entry.kind);
  const correlation = readCorrelation(entry.correlation);
  const agent = readAgent(entry.current_agent);

  body.appendChild(el("div", { class: "inspector-title" }, [
    el("span", { class: "mono strong", text: `#${entry.sequence}` }),
    channelTag(entry.channel),
    el("span", { class: "mono", text: kind.name }),
  ]));

  body.appendChild(block("Envelope and provenance", [
    el("dl", { class: "facts" }, [
      el("dt", { text: "Sequence" }),
      el("dd", { class: "mono", text: `${entry.sequence} — canonical append position` }),
      el("dt", { text: "Recorded at" }),
      el("dd", {}, [
        el("span", { class: "mono", text: entry.recorded_at }),
        el("span", { class: "muted", text: " descriptive only; not an order and not a duration" }),
      ]),
      el("dt", { text: "Schema" }),
      el("dd", { text: `${kind.schema} — ${kind.name}` }),
      el("dt", { text: "Adapter" }),
      el("dd", { class: "mono", text: entry.adapter }),
      el("dt", { text: "Mechanism" }),
      el("dd", { class: "mono", text: entry.mechanism }),
      el("dt", { text: "Correlation" }),
      el("dd", {}, correlation
        ? [el("span", { class: "mono", text: correlation.id }), el("span", { class: "muted", text: ` (${correlation.scheme})` })]
        : [el("span", { class: "muted", text: "this record carries no correlation id" })]),
      el("dt", { text: "Current agent" }),
      el("dd", { class: agent.state === "supplied" ? "mono" : "muted", text: agent.label }),
      el("dt", { text: "prompt_id" }),
      el("dd", {}, entry.prompt_id
        ? [el("span", { class: "mono", text: entry.prompt_id }),
           el("span", { class: "muted", text: " — carried as delivered. It groups nothing: what it delimits is unestablished." })]
        : [el("span", { class: "muted", text: "not supplied" })]),
    ]),
  ]));

  if (entry.facets.duration_ms !== null && entry.facets.duration_ms !== undefined) {
    body.appendChild(block("Supplied duration", [
      el("p", { class: "mono", text: `duration_ms ${entry.facets.duration_ms}` }),
      el("p", { class: "note", text: "As supplied by the integration on this record. Nothing here measures anything." }),
    ]));
  }
  if (entry.facets.interrupted !== null && entry.facets.interrupted !== undefined) {
    body.appendChild(block("Interruption", [
      el("p", { class: "mono", text: `interrupted: ${entry.facets.interrupted}` }),
    ]));
  }

  if (entry.subject_agent) {
    const subject = entry.subject_agent;
    body.appendChild(el("section", { class: "block" }, [
      el("h3", {}, [el("span", { text: "Lifecycle subject " }), derivedTag("distinct from emitter")]),
      el("p", { class: "note", text: "This record is about the agent below. It is not evidence that the record came from it." }),
      el("dl", { class: "facts" }, [
        el("dt", { text: "Subject agent" }),
        el("dd", { class: "mono", text: subject.agent_id }),
        el("dt", { text: "Subject type" }),
        el("dd", { text: subject.agent_type === null ? "not supplied" : subject.agent_type === "" ? "(empty string, as delivered)" : subject.agent_type }),
        el("dt", { text: "Parent" }),
        el("dd", {
          class: subject.supplied_parent ? "mono" : "muted alarm",
          text: subject.supplied_parent
            ? `${subject.supplied_parent.agent_id || "(no id)"} ${subject.supplied_parent.agent_type || ""}`.trim()
            : "parent identity not supplied — none is inferred from containment, adjacency, or timing",
        }),
      ]),
    ]));
  }

  const anomalies = entryAnomalies(entry.sequence);
  if (anomalies.length) {
    body.appendChild(el("section", { class: "block block-alarm" }, [
      el("h3", {}, [el("span", { text: "Anomalies citing this record " }), derivedTag()]),
      ...anomalies.map((anomaly) => {
        const read = readAnomaly(anomaly);
        return el("div", { class: "anomaly-row" }, [
          el("span", { class: "anomaly-name", text: read.text }),
          receiptList(anomaly.receipts),
        ]);
      }),
    ]));
  }

  if (correlation) {
    const group = state.groupsByKey.get(correlation.key);
    if (group) {
      const inner = group.evidence.v1 || group.evidence.v2 || {};
      const sections = [
        el("p", { class: "note", text: "Correlation places evidence beside evidence. These are separate records making separate claims; nothing here is merged into one step." }),
        el("div", { class: "count-row" }, [
          el("span", { class: "count-label", text: "shape" }),
          derivedTag(SHAPE_TEXT[group.shape] || group.shape),
        ]),
      ];
      if (group.paired_interval) {
        sections.push(el("div", { class: "count-row" }, [
          el("span", { class: "count-label", text: "positions" }),
          receiptButton(group.paired_interval.opening),
          el("span", { class: "muted", text: "→" }),
          receiptButton(group.paired_interval.outcome),
        ]));
        sections.push(el("p", { class: "note", text: "Two positions in the append chain. Not elapsed time, not execution duration, not nesting, not containment." }));
      }
      if (group.reported_intents.records.length) {
        sections.push(el("h4", {}, [el("span", { text: "Reported " }), channelTag("reported")]));
        for (const sequence of group.reported_intents.records) {
          const other = state.index.get(sequence);
          if (other) sections.push(evidenceCard(other, "a claim"));
        }
      }
      const observed = [];
      for (const [field, list] of Object.entries(inner)) {
        for (const sequence of list) observed.push([sequence, OBSERVED_LABELS[field] || field]);
      }
      observed.sort((a, b) => a[0] - b[0]);
      if (observed.length) {
        sections.push(el("h4", {}, [el("span", { text: "Observed " }), channelTag("observed")]));
        for (const [sequence, label] of observed) {
          const other = state.index.get(sequence);
          if (other) sections.push(evidenceCard(other, label));
        }
      } else {
        sections.push(el("p", { class: "count-absent", text: "no observed record carries this id" }));
      }
      if (group.delivered_tool_names.length > 1) {
        sections.push(el("p", { class: "warn-note", text: `Records for this id delivered different tool names: ${group.delivered_tool_names.map((d) => d.value).join(", ")}. None is canonical.` }));
      }
      sections.push(el("p", { class: "count-scope", text: scopeLabel(group.scope) }));
      body.appendChild(el("section", { class: "block" }, [
        el("h3", {}, [el("span", { text: "Correlated evidence " }), derivedTag()]),
        ...sections,
      ]));
    }
  }

  body.appendChild(block("Raw record", [
    el("p", { class: "note", text: "Exactly as it appears in the recording. Rendered as text throughout." }),
    collapsedPayload("full record envelope and event data", entry.record),
  ]));
}

// ---------------------------------------------------------------------------
// Coverage — what was and was not captured
// ---------------------------------------------------------------------------

function renderCoverage() {
  const p = state.projection;
  const body = document.getElementById("coverage-body");
  clear(body);

  body.appendChild(el("p", { class: "lede", text: "What this recording did and did not capture. Every count is a statement about records observed, never about events that occurred, and every zero carries the population it was counted in." }));

  const gaps = [];
  for (const coverage of p.coverage) {
    if (!coverage.population.records.length) continue;
    const supplied = coverage.present.records.length;
    const total = coverage.population.records.length;
    const row = el("div", { class: "gap-row" }, [
      el("span", { class: "gap-label", text: COVERAGE_TEXT[coverage.field] || coverage.field }),
      el("span", {
        class: supplied === 0 ? "alarm" : "",
        text: supplied === 0 ? `never supplied, on any of ${total} record(s)` : `supplied on ${supplied} of ${total} record(s)`,
      }),
    ]);
    if (coverage.absent.records.length) {
      row.appendChild(el("span", { class: "muted", text: "absent on:" }));
      row.appendChild(receiptList(coverage.absent.records));
    }
    gaps.push(row);
  }
  for (const tally of p.aggregates.by_event_kind) {
    if (tally.records.records.length) continue;
    const kind = readKind(tally.value);
    gaps.push(el("div", { class: "gap-row" }, [
      el("span", { class: "gap-label", text: kind.name }),
      el("span", { class: "alarm", text: `no ${kind.name} record observed` }),
      el("span", { class: "count-scope", text: scopeLabel(tally.records.scope) }),
    ]));
  }
  gaps.push(el("p", { class: "note", text: "A surface this recording did not exercise is not a working surface and is not shown as one. Two silences agreeing is not corroboration." }));
  gaps.push(el("p", { class: "note", text: "Tool events do not reveal every file a session changed: a command that writes a file produces no mutation event. Nothing here is a complete account of what changed." }));
  body.appendChild(block("Evidence gaps", gaps, { class: "block-alarm" }));

  body.appendChild(block("Anomalies", p.anomalies.length
    ? p.anomalies.map((anomaly) => {
        const read = readAnomaly(anomaly);
        return el("div", { class: "anomaly-row" }, [
          el("span", { class: "anomaly-name", text: read.text }),
          read.subject ? el("span", { class: "mono muted", text: read.subject }) : null,
          anomaly.receipts.length
            ? receiptList(anomaly.receipts)
            : el("span", { class: "count-scope", text: scopeLabel(anomaly.scope) }),
        ]);
      })
    : [el("p", { class: "note", text: "No anomaly was detected in the records examined." })],
    { derived: true }));

  body.appendChild(block("Event kinds", [
    el("div", { class: "counts" }, p.aggregates.by_event_kind.map((tally) => {
      const kind = readKind(tally.value);
      return countRow(kind.name, tally.records, `no ${kind.name} record observed`);
    })),
    el("p", { class: "note", text: "The whole schema vocabulary, including kinds this recording contains none of." }),
  ]));

  if (p.subagents.length) {
    body.appendChild(block("Subagent boundaries", p.subagents.map((subagent) => {
      const types = subagent.delivered_types.map((d) =>
        d.value === null ? "(no type supplied)" : d.value === "" ? "(empty string)" : d.value);
      return el("div", { class: "subagent-row" }, [
        el("span", { class: "mono strong", text: subagent.agent_id }),
        el("span", { class: "pair" }, [
          el("span", { text: "start " }),
          subagent.started.records.length
            ? receiptList(subagent.started.records)
            : el("span", { class: "count-absent", text: "start not observed" }),
        ]),
        el("span", { class: "pair" }, [
          el("span", { text: "stop " }),
          subagent.stopped.records.length
            ? receiptList(subagent.stopped.records)
            : el("span", { class: "count-absent", text: "stop not observed" }),
        ]),
        el("span", { class: "muted", text: `type(s) delivered: ${types.join(", ")}` }),
        subagent.supplied_parents.length
          ? el("span", { class: "muted", text: `parent supplied: ${subagent.supplied_parents.map((d) => d.value.agent_id || "(type only)").join(", ")}` })
          : el("span", { class: "count-absent", text: "parent identity not supplied — no parentage is inferred" }),
      ]);
    })));
  }
}

// ---------------------------------------------------------------------------
// Provenance — where this recording came from
// ---------------------------------------------------------------------------

function renderProvenance() {
  const p = state.projection;
  const body = document.getElementById("provenance-body");
  clear(body);

  body.appendChild(el("p", { class: "lede", text: "Where this recording came from, and what its capture points could see." }));

  const truncated = isTruncated(p.scope);
  body.appendChild(block("Recording", [
    el("dl", { class: "facts" }, [
      el("dt", { text: "Session" }),
      el("dd", { class: "mono", text: p.session_id === null ? "no complete record, so no session" : p.session_id }),
      el("dt", { text: "Schema" }),
      el("dd", { text: p.schema_version === null ? "no complete record, so no schema version" : `v${p.schema_version}` }),
      el("dt", { text: "Records" }),
      el("dd", { text: String(p.records.length) }),
      el("dt", { text: "Completeness" }),
      el("dd", {
        class: truncated ? "alarm" : "",
        text: truncated
          ? `ends mid-record — ${p.scope.valid_prefix.fragment_bytes} byte fragment at byte ${p.scope.valid_prefix.fragment_byte_offset}, not replayed`
          : "complete — the final record was newline-terminated",
      }),
    ]),
  ]));

  if (p.timestamps) {
    const t = p.timestamps;
    const rows = [
      el("dl", { class: "facts" }, [
        el("dt", { text: "Earliest" }),
        el("dd", {}, [el("span", { class: "mono", text: t.earliest.recorded_at }), receiptButton(t.earliest.sequence)]),
        el("dt", { text: "Latest" }),
        el("dd", {}, [el("span", { class: "mono", text: t.latest.recorded_at }), receiptButton(t.latest.sequence)]),
      ]),
      el("p", { class: "note", text: "Recorder wall clock, descriptive only. It establishes no order, duration, overlap, or causality — append sequence is the only order." }),
    ];
    if (t.non_monotonic.records.length) {
      rows.push(el("p", { class: "warn-note" }, [
        el("span", { text: `The recorder's clock moved backwards at ${t.non_monotonic.records.length} record(s). Append order is unaffected. ` }),
        receiptList(t.non_monotonic.records),
      ]));
    }
    body.appendChild(block("Recorder time", rows, { derived: true }));
  }

  body.appendChild(block("Channels", [
    el("div", { class: "counts" }, p.aggregates.by_channel.map((tally) =>
      el("div", { class: "count-row" }, [
        channelTag(tally.value),
        el("span", { class: "count-value", text: String(tally.records.records.length) }),
        tally.records.records.length === 0
          ? el("span", { class: "count-absent", text: "no record observed on this channel" })
          : null,
      ]))),
    el("p", { class: "note", text: "Raw provenance: how a record reached the recording. Derived claims are not a channel." }),
  ]));

  body.appendChild(block("Capture points", [
    el("div", { class: "counts" }, [
      ...p.aggregates.by_adapter.map((t) => countRow(`adapter: ${t.value}`, t.records)),
      ...p.aggregates.by_mechanism.map((t) => countRow(t.value, t.records)),
    ]),
  ]));

  const agentRows = p.current_agents.supplied.map((t) => countRow(t.value, t.records));
  agentRows.push(countRow("identity not supplied", p.current_agents.not_supplied, "every record supplied an identity"));
  if (p.current_agents.not_representable.records.length) {
    agentRows.push(countRow("no causal context in this schema", p.current_agents.not_representable));
  }
  body.appendChild(block("Records by supplied agent identity", [
    el("div", { class: "counts" }, agentRows),
    el("p", { class: "note", text: "Counts records by the identity they were delivered from. An absent identity is not supplied — it is not a root agent, and it is not the same as a subagent boundary's subject." }),
  ]));
}

// ---------------------------------------------------------------------------
// Selection and perspectives
// ---------------------------------------------------------------------------

/**
 * Selection changes styling, not structure.
 *
 * Rebuilding the ledger here would destroy the row the reader is standing on,
 * which sends focus back to the document and makes arrow-key navigation stop
 * after a single step. Filters and search rebuild; selection does not.
 */
function applySelection() {
  const wanted = String(state.selected);
  for (const row of document.querySelectorAll("#ledger-body tbody tr")) {
    const on = row.getAttribute("data-sequence") === wanted;
    row.classList.toggle("row-selected", on);
    row.setAttribute("aria-selected", String(on));
  }
  for (const mark of document.querySelectorAll(".mark")) {
    mark.classList.toggle("mark-selected", mark.getAttribute("data-sequence") === wanted);
  }
  updateMarkTabStop();
}

function select(sequence, options = {}) {
  state.selected = sequence;
  if (options.reveal && state.perspective !== "events") showPerspective("events");
  applySelection();
  renderInspector();
  const row = document.querySelector(`#ledger-body tr[data-sequence="${CSS.escape(String(sequence))}"]`);
  if (!row) return;
  if (options.keepFocus) return;
  row.scrollIntoView({ block: "nearest" });
  if (options.reveal) row.focus();
}

const PERSPECTIVES = ["events", "coverage", "provenance"];

function showPerspective(name, options = {}) {
  state.perspective = name;
  for (const other of PERSPECTIVES) {
    const tab = document.getElementById(`tab-${other}`);
    const panel = document.getElementById(`panel-${other}`);
    const on = other === name;
    tab.setAttribute("aria-selected", String(on));
    tab.setAttribute("tabindex", on ? "0" : "-1");
    panel.hidden = !on;
  }
  if (options.focus) document.getElementById(`tab-${name}`).focus();
  // Marks are only measurable while their panel is visible.
  if (name === "events") updateMarkTabStop();
}

function wireTabs() {
  const tabs = PERSPECTIVES.map((name) => document.getElementById(`tab-${name}`));
  for (const tab of tabs) {
    tab.addEventListener("click", () => showPerspective(tab.getAttribute("data-perspective")));
    tab.addEventListener("keydown", (event) => {
      const at = tabs.indexOf(tab);
      let next = null;
      if (event.key === "ArrowRight") next = tabs[(at + 1) % tabs.length];
      else if (event.key === "ArrowLeft") next = tabs[(at - 1 + tabs.length) % tabs.length];
      else if (event.key === "Home") next = tabs[0];
      else if (event.key === "End") next = tabs[tabs.length - 1];
      if (!next) return;
      event.preventDefault();
      showPerspective(next.getAttribute("data-perspective"), { focus: true });
    });
  }
}

function wireControls() {
  for (const radio of document.querySelectorAll('input[name="axis"]')) {
    radio.addEventListener("change", (event) => {
      if (!event.target.checked) return;
      state.axis = event.target.value;
      renderMap();
      applySelection();
    });
  }

  const search = document.getElementById("search-input");
  search.addEventListener("input", () => {
    state.search = search.value.trim();
    renderLedger();
    renderMap();
    applySelection();
  });

  const anomalous = document.getElementById("filter-anomalous");
  anomalous.addEventListener("change", () => {
    state.filters.anomalous = anomalous.checked;
    onFiltersChanged();
  });

  const payloads = document.getElementById("search-payloads");
  const note = document.getElementById("search-note");
  payloads.addEventListener("change", () => {
    state.searchPayloads = payloads.checked;
    note.textContent = payloads.checked
      ? "Payload search is ON. This reads commands, file contents, and tool output — the most sensitive part of the recording."
      : "Searching metadata only. Payload search reads commands, file contents, and tool output — the most sensitive part of the recording — and is off until you turn it on.";
    note.classList.toggle("warn-note", payloads.checked);
    renderLedger();
    renderMap();
    applySelection();
  });
}

async function main() {
  const failure = document.getElementById("failure");
  try {
    const response = await fetch(`/projection.json?c=${encodeURIComponent(CAPABILITY)}`, {
      cache: "no-store",
      credentials: "omit",
      referrerPolicy: "no-referrer",
    });
    if (!response.ok) throw new Error(`the snapshot endpoint answered ${response.status}`);
    state.projection = await response.json();
  } catch (error) {
    failure.hidden = false;
    failure.textContent = `Could not load the snapshot: ${error.message}. This page must be opened through the URL the command printed, capability included.`;
    return;
  }

  buildIndexes();
  document.getElementById("workbench").hidden = false;
  renderSummary();
  renderFilters();
  renderActiveFilters();
  renderLedger();
  renderMap();
  renderInspector();
  renderCoverage();
  renderProvenance();
  wireTabs();
  wireControls();
  showPerspective("events");
}

main();
