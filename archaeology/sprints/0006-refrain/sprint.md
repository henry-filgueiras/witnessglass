---
id: spr_01KZ7A9ZHPYSQH7WATWCZXA133
sequence: 6
kind: sprint
status: active
created: 2026-08-04
---

# Refrain

## Goal

Find out whether the multiresolution evidence sprint:5 produced can actually constrain Matrix
Profile's window parameter — not whether a Matrix Profile can be made to run.

Two questions, and the second is the one that can embarrass us:

> Can Haar-derived scale evidence reduce Matrix Profile's window-selection arbitrariness enough to
> recover known repeated behaviour in sparse synthetic WitnessGlass signals and identify plausible
> recurring or discordant regions in a real recording?

> Does choosing windows near Haar-indicated scales actually outperform windows that Haar evidence
> would make us less likely to choose?

## Rationale

sprint:5 recommended changepoint detection and this sprint does something else, on instruction and
for a reason worth writing down: the Haar round produced a *parameter-selection story* — coarse
scales carry the structure, fine scales are indistinguishable from sparsity — and a story of that
shape is exactly the kind this project has twice been wrong about. sprint:3 exists because a chain
of individually reasonable steps produced a false conclusion inside one system asking itself whether
it was complete. A satisfying multiresolution narrative used to pick a Matrix Profile window is the
same shape of risk, and the only way to find out is to run the control.

**Haar scale and Matrix Profile window are not the same quantity, and sprint:5's language did not
always keep them apart.** A Haar level is a *contrast between adjacent means*, and its diagnostic
for a periodic train is a cliff at the level whose half-window reaches the period. A Matrix Profile
window is the *length of the pattern being matched*. The injected motif has three separate numbers
attached to it — an instance duration of about 1–2 s, a repetition period of 8 s, and a Haar cliff
at the 16 s level — and there is no reason to expect a good Matrix Profile window to coincide with
any particular one of them. This sprint has to say which it is testing at every point.

There is also a specific way this could all be moot, and it is predictable in advance rather than
after. These signals are 78–94% empty. `motif-rs` documents its convention for constant
subsequences — two constant subsequences are at distance exactly 0 — so in a mostly-empty series
the global minimum of the matrix profile is 0, attained by two stretches of nothing. Any result
read off an unmasked profile would be a statement about emptiness. That is a representation problem
rather than a bug, and if it dominates, the finding is that sampled univariate Matrix Profile is the
wrong shape for this data.

## Success criteria

- An implementation decision made on inspected evidence — license, maturity, dependency footprint,
  validation against a reference — rather than on convenience, and recorded with what was
  inspected.
- The library's behaviour on exclusion zones, constant subsequences, normalization, and
  index-to-time conversion pinned by tests, so a later reader is protected from *our*
  misunderstanding of it as much as from its defects.
- A window ladder of four to six lengths, derived from committed evidence and written down with its
  predictions **before the detector is run on any fixture**, including at least one control window
  drawn from the region sprint:5 found indistinguishable from an impulse null.
- A null that answers "what does a good motif distance look like when temporal order is destroyed",
  and a comparison metric fixed in advance rather than chosen once the numbers are visible.
- Per-dimension independence, as in both prior rounds. No multivariate fusion.
- An honest three-way verdict on the guidance hypothesis — supported, mixed, or falsified — kept
  separate from any verdict on Matrix Profile itself.
- Any representation failure recorded as a finding rather than patched around.

## Non-goals

- **Changepoint detection.** sprint:5 recommended it; it is still unimplemented and this sprint does
  not touch it.
- Multivariate or multidimensional Matrix Profile. The library ships `mstump` and `mmotifs`; this
  sprint uses neither. If independent dimensions turn out to lose essential structure, that is a
  finding and a later task's authorization, not something to reach for mid-round.
- Implementing STOMP, STAMP, STAMPI, or SCRIMP by hand if an adequate implementation exists.
- A detector trait, a plugin system, a registry, or any production analytics surface.
- Any change to the raw format, the schema, the recorder, `inspection`, or the viewer; any product
  CLI surface; any web UI work; any Python runtime or repository dependency.
- Committing, copying, or depending on a real recording.
- Rewriting sprint:5's conclusions. Its result stands as written, including the recommendation this
  sprint declines to follow yet.
