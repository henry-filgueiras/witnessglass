---
id: spr_01KZ77XQGFN2C3CD0WWESMAT98
sequence: 5
kind: sprint
status: closed
created: 2026-08-04
closed: 2026-08-04
---

# Octave

## Goal

Find out whether a simple Haar multiresolution decomposition exposes deliberately injected
temporal structure in a WitnessGlass behavioral signal — including when that signal is as sparse as
a real recording, rather than as sparse as a fixture built to be legible.

This is sprint:4's recommended next step, taken with one correction to sprint:4's own reasoning,
recorded below. It runs one detector and earns or rejects the next.

## Rationale

sprint:4 built the substrate and validated it against an oracle whose structure was known in
advance. It closed with three empirical results and a recommendation, and the recommendation's
stated reason was partly wrong.

The three results stand:

1. a real untracked 234-record session is ~94% empty at 500 ms and still ~22% empty at 30 s;
2. `recorded_response_json_bytes` is violently heavy-tailed — mean 161, standard deviation 1349,
   maximum 23936, about 17.6 standard deviations above the mean in a single bucket;
3. the legible oracle is substantially denser than reality — 78% empty against 94%.

### The correction

task:14 §8 recommended Haar on the grounds that it is "the one of the three that *answers* the
width question instead of presupposing it". That overstates it, and the overstatement is the kind
this project is supposed to catch.

**A Haar transform does not eliminate the need to choose a sampling interval. It operates on an
already-sampled signal.** It cannot see structure below the interval it was given, and the choice
of 500 ms remains exactly as much a modeling decision after the transform as before it.

What it actually offers is narrower and still worth having: given a sufficiently fine sampling
interval, it decomposes the signal across dyadic temporal scales, and the distribution of energy
across those scales is *evidence about which scales carry behavior*. That evidence can inform a
later choice of Matrix Profile subsequence length or of a coarser aggregation. It is not a
derivation of an optimal bucket width and this sprint must not describe it as one.

### Why this is worth running at all

Because there is a real possibility the answer is no, and it is cheap to find out. A signal that is
94% empty is a train of isolated impulses, and an isolated impulse has a known Haar signature that
has nothing to do with behaviour: its energy halves at every coarser level, regardless of what the
agent was doing. If every dimension of every recording produces that decay and nothing else, the
transform has found the sparsity and not the session, and this line of work should stop.

So the experiment is set up to be falsifiable against that null rather than against nothing.

## Success criteria

- A Haar decomposition small enough to read in one sitting, with its normalization convention
  stated, its treatment of non-power-of-two lengths chosen deliberately and documented, and an
  exact energy identity that holds as a test rather than as a claim.
- A second synthetic fixture whose emptiness at 500 ms is in the range the real recording
  established, carrying known injected structure, and explicitly labelled as the stress case
  against the existing legible best case. The existing oracle is unchanged.
- Predictions written down before the transform is run, including the isolated-impulse null the
  results must be read against, and not revised afterwards.
- Each licensed dimension decomposed independently. No multivariate fusion scheme, and no
  aggregation across dimensions that would let one dimension's magnitude reach another's result.
- The heavy-tailed dimension investigated rather than repaired: the analysis run with and without
  it, and the question of whether it contaminates anything answered with a measurement.
- A recommendation of exactly one next experiment, with the empirical reason attached, including
  "stop" if the evidence supports stopping.

## Non-goals

- **Matrix Profile and changepoint detection.** Both are the thing this sprint exists to earn or
  reject. Neither is implemented here, at any size.
- Daubechies, Symlets, generic filter banks, an inverse transform, a wavelet framework, SIMD, or
  any wavelet other than Haar.
- Adopting any change to sprint:4's normalization policy. Evidence for or against one may be
  collected; the change itself needs its own adjudication.
- Semantic interpretation of a peak. "This dimension carries energy near an 8 s scale" is
  evidence; "the agent has an 8 s loop" is an interpretation this project has no license to make,
  and the distinction is not softened anywhere in the output.
- Any change to the raw format, the schema, the recorder, `inspection`, or the viewer; any new
  dependency; any product CLI surface; any web UI work.
- Committing, copying, or depending on a real recording.

## Outcome

One task, closed. **The answer to the sprint's question is yes**, on both fixtures and with room to
spare: a simple Haar decomposition exposes the injected structure, distinguishes the two kinds of
injected structure from each other, and distinguishes both from the isolated-impulse decay that a
mostly-empty recording produces on its own. The falsification condition fixed in advance was not
met and not narrowly — observed ratios to the null span 0.02 to 34.4 where the stop condition was
±25% of 1.

The correction this sprint's rationale made to sprint:4's reasoning held up in use. Haar did not
answer the sampling-width question and was never asked to. It operated on an already-sampled
signal, could not see below it, and produced evidence about which scales carry behaviour — which
is what changed the next recommendation, and is a narrower and more defensible thing than sprint:4
claimed for it.

### Success criteria, against evidence

- **A readable transform with a stated convention, a deliberate odd-length policy, and an exact
  identity.** One file, no dependency, orthonormal Haar, both scale readings carried so neither has
  to be inferred. `input = detail + approximation + remainders` holds across every length from 0 to
  64 and every dimension of both fixtures; worst residual on a real projection `4.8e-10`. Odd tails
  are set aside as recorded remainders rather than padded against or discarded, and the three
  rejected alternatives are written down beside the one chosen.

- **A stress fixture at observed density.** 365 records, 2401 buckets, **92.7% empty** against the
  legible oracle's 78.2% and the real session's 94.4%, with the band asserted by a test. Its motif
  period is dyadic and its regime block deliberately is not, so the fixture does not consist
  entirely of structure that flatters one transform.

- **Predictions recorded before the run and not revised.** Six, written into task:15 before the
  transform existed in runnable form. Five held; the sixth is recorded as falsified.

- **Per-dimension independence.** Every column decomposed on its own. The with/without comparison
  for the heavy-tailed dimension produced bit-identical spectra for every other dimension —
  computed and compared, not assumed from the architecture.

- **The heavy-tailed dimension investigated rather than repaired.** No policy changed. What the
  round found instead is stronger than a policy: detail coefficients are offset-invariant and scale
  linearly, so energy shares are invariant to both, and sprint:4's z-score cannot move a single
  share — measured at `7.2e-16`. The normalization question is *orthogonal* to scale-spectrum work
  and can be adjudicated later on its own evidence.

- **One next experiment recommended, with its reason.** Changepoint detection, on the evidence that
  the real recording carries the block signature and not the periodic one.

### What the sprint found that it was not looking for

**A period shows as a cliff, not as a peak.** The naive expectation — energy peaks at the scale
matching the period — is wrong, and a reader holding it would have concluded there was no
periodicity in a fixture built to contain nothing else. Once a half-window reaches the period, both
halves hold equal numbers of instances and the contrast cancels. The period is the last level
*before* the collapse. The same logic explains the block signature: a window inside a regime
cancels, and the window spanning its edge — around twice the block width — is where the excess
lands.

**The odd-length policy costs the end of a recording, and it cost a whole dimension here.** A
remainder is excluded from the level that set it aside and from every coarser one, so a level-1
remainder reaches no level at all. Both fixtures have odd sample counts; both put their final base
sample in the level-1 remainder; and in both, that sample is the only one `kind:v2:session_ended`
has. It decomposes to zero everywhere. This was found by running, not by predicting, and the fix
was to make the transform report it as distinct from a genuinely flat dimension — because a reader
who cannot tell those apart reads an artefact of the transform as a property of the recording.
Coverage per level is now printed for the same reason: the legible oracle's coarsest level is
computed over 53% of the recording.

**A sparser recording was not a harder one.** Prediction 6 said the sparse fixture would show
weaker signatures; it showed sharper ones. Density was not the limiting factor. Structural
isolation within a dimension, and recording length — the sparse fixture is five times longer and
reaches five more dyadic levels — mattered more. sprint:4's framing of sparsity as the central
obstacle was too simple, and this is the evidence against it.

**500 ms is unnecessarily fine for whole-session analysis, now with a measurement.** On the real
recording, levels 1 through 5 — 1 s to 16 s — sit between 0.7 and 1.6 against the null across
almost every dimension. Below roughly 32 s that recording is, to this transform, indistinguishable
from its own emptiness. That is a statement about what was found and not a claim that nothing is
there, and it says the *analysis* should resample while the substrate default should stay where it
is.

### What this sprint deliberately leaves open

Whether a changepoint detector recovers the regime boundaries both oracles contain, and whether it
finds anything in a real recording. That is the next sprint's question.

Whether any real recording carries a periodic motif at all. This one does not, on this evidence,
and one session is one session. If a second shows the periodic cliff, Matrix Profile earns its turn
then; nothing here rules it out permanently.

sprint:4's normalization policy stands, unexamined and now known to be irrelevant to this class of
result. Any change still needs its own adjudication.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the viewer, or the
product CLI, and no dependency was added.
