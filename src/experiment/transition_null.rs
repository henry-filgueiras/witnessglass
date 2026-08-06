//! **Disposable research experiment.** sprint:20, task:30.
//!
//! Two first-order categorical nulls, and the measurements that decide which of
//! them may be called transition-preserving.
//!
//! # Why this module exists
//!
//! sprint:19 calibrated the complete search against an *order* null — marks
//! permuted across the whole sequence — and then bounded its own result. Every
//! observed recording has an immediate repetition rate of exactly zero and a
//! mean run length of exactly one; the order null produces repeats constantly.
//! The recordings sit outside the entire null range on the most trivial local
//! statistic there is, so an exceptional `T` under that null rejects
//! exchangeable ordering and establishes nothing about motifs.
//!
//! The repair sprint:19 named is a null that keeps each recording's own
//! first-order transition structure and destroys everything longer. This module
//! builds two of them, because the obvious construction and the honest
//! description of it do not match:
//!
//! - [`markov_null_seeded`] fits a first-order chain to the sequence and
//!   resamples from it. Transition frequencies and mark marginals are preserved
//!   **only in expectation**, and at a 32-event recording that is a long way
//!   from preserved.
//! - [`doublet_null_seeded`] shuffles the sequence so that **every first-order
//!   transition count is preserved exactly**, along with the mark multiset and
//!   both endpoints. It is a permutation of the observed marks, exactly as the
//!   order null is, and reaches a different set of permutations.
//!
//! Which one a round may honestly call "transition-preserving" is decided by
//! measurement in [`fidelity`], before any criterion about `T` is written.
//!
//! # What neither of them is allowed to be
//!
//! Domain-neutral. Both consume categorical marks and sequence order and
//! nothing else — no schema knowledge, no tool semantics, no timing feature, no
//! path. Both leave gaps and offsets attached to **positions**, exactly as
//! [`super::event_sequence::order_null_seeded`] does, so the only property that
//! changes between sprint:19's null and this round's is the mark process.
//! Receipts are dropped: a moved mark is not that record's mark.

use std::collections::BTreeMap;

use serde::Serialize;

use super::event_sequence::{EventSequence, Mark, order_null_seeded};

/// One null construction: a sequence and a seed in, a replacement sequence out.
pub type Construction = for<'a> fn(&EventSequence<'a>, u64) -> EventSequence<'a>;

/// The three constructions this round compares, in the order they are reported.
///
/// `order` is sprint:19's, unchanged and re-run so the two rounds are paired.
/// The other two are this round's, and which of them may be called
/// transition-preserving is settled by measurement, not by naming.
pub const CONSTRUCTIONS: [(&str, Construction); 3] = [
    ("order", order_null_seeded),
    ("doublet", doublet_null_seeded),
    ("markov", markov_null_seeded),
];

/// A fixed linear congruential generator, so a null is reproducible.
///
/// The same shape and the same constants as
/// [`super::event_sequence`]'s own, mirrored here rather than exported so that
/// this round adds no generator and touches no line of the search.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_below(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        if bound == 0 {
            0
        } else {
            ((self.0 >> 33) as usize) % bound
        }
    }

    /// Fisher–Yates, in place.
    fn shuffle<T>(&mut self, items: &mut [T]) {
        for index in (1..items.len()).rev() {
            let pick = self.next_below(index + 1);
            items.swap(index, pick);
        }
    }
}

// ---------------------------------------------------------------------------
// The state space
// ---------------------------------------------------------------------------

/// The distinct marks of a sequence, in first-appearance order, with each
/// event's index into them.
///
/// **The whole state space of both nulls.** A state is a delivered mark and
/// nothing else: no category, no grouping, no semantic class. Marks are compared
/// by [`Mark`]'s own equality, so a reported mark is never equal to an observed
/// one here any more than it is anywhere else.
pub fn states<'a>(sequence: &EventSequence<'a>) -> (Vec<Mark<'a>>, Vec<usize>) {
    let mut vocabulary: Vec<Mark<'a>> = Vec::new();
    let mut indices: Vec<usize> = Vec::with_capacity(sequence.events.len());
    for event in &sequence.events {
        let index = match vocabulary.iter().position(|mark| *mark == event.mark) {
            Some(found) => found,
            None => {
                vocabulary.push(event.mark);
                vocabulary.len() - 1
            }
        };
        indices.push(index);
    }
    (vocabulary, indices)
}

/// Rewrite a sequence's marks from a state path, leaving everything else where
/// it was.
///
/// Gaps, offsets, length, session identity and the timing skeleton are
/// untouched; every receipt is dropped, because a mark that moved is not the
/// mark that record carried.
fn with_marks<'a>(
    sequence: &EventSequence<'a>,
    vocabulary: &[Mark<'a>],
    path: &[usize],
) -> EventSequence<'a> {
    let mut replaced = sequence.clone();
    for (event, state) in replaced.events.iter_mut().zip(path) {
        event.mark = vocabulary[*state];
        event.sequence = None;
    }
    replaced
}

// ---------------------------------------------------------------------------
// The exact construction — first-order counts preserved by permutation
// ---------------------------------------------------------------------------

/// **The exact first-order null.** A permutation of the observed marks whose
/// every first-order transition count is identical to the observed one.
///
/// The sequence is read as a walk on a multigraph whose vertices are marks and
/// whose edges are the observed adjacent pairs. Any Eulerian trail of that
/// multigraph starting at the observed first mark is a sequence with exactly the
/// observed doublet counts, exactly the observed mark counts, and exactly the
/// observed length. This draws one:
///
/// 1. every vertex but the terminal one nominates one outgoing edge to be used
///    last, drawn uniformly from its outgoing edges;
/// 2. the nominations are accepted only when following them from every vertex
///    reaches the terminal vertex — that is, when they form a spanning
///    arborescence rooted there — and are redrawn otherwise;
/// 3. each vertex's remaining edges are shuffled and the nominated edge appended;
/// 4. the trail is walked from the observed first mark, always taking the next
///    unused edge.
///
/// Step 2 is what makes the walk complete rather than strand itself, and drawing
/// uniformly with rejection makes the accepted arborescence uniform among the
/// valid ones. This is the doublet-preserving shuffle of Altschul and Erickson,
/// as corrected by Kandel and colleagues; nothing about it is specific to this
/// project or to any recording.
///
/// Returns the observed sequence unchanged, with receipts dropped, when the
/// sequence is too short to have a transition or when step 2 does not converge
/// inside [`ARBORESCENCE_ATTEMPTS`]. Both are counted rather than hidden:
/// see [`degeneracy`].
pub fn doublet_null_seeded<'a>(sequence: &EventSequence<'a>, seed: u64) -> EventSequence<'a> {
    let (vocabulary, path) = states(sequence);
    let Some(shuffled) = doublet_path(&path, vocabulary.len(), seed) else {
        return with_marks(sequence, &vocabulary, &path);
    };
    with_marks(sequence, &vocabulary, &shuffled)
}

/// How many times the arborescence draw is retried before the construction
/// gives up and returns the observed path.
///
/// Fixed before execution. A failure is reported by [`degeneracy`] rather than
/// worked around, because a null that silently returns its own input is a null
/// that cannot separate from anything.
pub const ARBORESCENCE_ATTEMPTS: usize = 200;

/// The state-path half of [`doublet_null_seeded`], separated so it can be
/// tested against transition counts directly.
pub fn doublet_path(path: &[usize], vertices: usize, seed: u64) -> Option<Vec<usize>> {
    if path.len() < 3 || vertices == 0 {
        return None;
    }
    let terminal = *path.last()?;

    // The multigraph: one edge per observed adjacent pair.
    let mut outgoing: Vec<Vec<usize>> = vec![Vec::new(); vertices];
    for pair in path.windows(2) {
        outgoing[pair[0]].push(pair[1]);
    }

    let mut generator = Lcg::new(seed);
    for _ in 0..ARBORESCENCE_ATTEMPTS {
        // 1. Every vertex but the terminal one nominates a last edge.
        let mut nominated: Vec<Option<usize>> = vec![None; vertices];
        for (vertex, edges) in outgoing.iter().enumerate() {
            if vertex == terminal || edges.is_empty() {
                continue;
            }
            nominated[vertex] = Some(generator.next_below(edges.len()));
        }

        // 2. Accept only when every nomination leads to the terminal vertex.
        if !reaches_terminal(&nominated, &outgoing, terminal, vertices) {
            continue;
        }

        // 3. Order each vertex's edges: the rest shuffled, the nominated one last.
        let mut ordered: Vec<Vec<usize>> = Vec::with_capacity(vertices);
        for (vertex, edges) in outgoing.iter().enumerate() {
            let mut rest = edges.clone();
            let last = nominated[vertex].map(|index| rest.remove(index));
            generator.shuffle(&mut rest);
            if let Some(edge) = last {
                rest.push(edge);
            }
            ordered.push(rest);
        }

        // 4. Walk the trail from the observed first mark.
        let mut used = vec![0usize; vertices];
        let mut current = path[0];
        let mut walked = Vec::with_capacity(path.len());
        walked.push(current);
        let mut complete = true;
        for _ in 1..path.len() {
            let edges = &ordered[current];
            if used[current] >= edges.len() {
                complete = false;
                break;
            }
            let next = edges[used[current]];
            used[current] += 1;
            walked.push(next);
            current = next;
        }
        if complete {
            return Some(walked);
        }
    }
    None
}

/// Whether following the nominated edges from every vertex that has one reaches
/// `terminal` without cycling.
fn reaches_terminal(
    nominated: &[Option<usize>],
    outgoing: &[Vec<usize>],
    terminal: usize,
    vertices: usize,
) -> bool {
    for start in 0..vertices {
        if start == terminal || nominated[start].is_none() {
            continue;
        }
        let mut current = start;
        let mut steps = 0;
        while current != terminal {
            let Some(index) = nominated[current] else {
                return false;
            };
            current = outgoing[current][index];
            steps += 1;
            if steps > vertices {
                return false;
            }
        }
    }
    true
}

// ---------------------------------------------------------------------------
// The in-expectation construction — a fitted chain, resampled
// ---------------------------------------------------------------------------

/// **The in-expectation first-order null.** Fit a first-order chain to the
/// sequence's own marks by maximum likelihood and generate a fresh path of the
/// same length from it.
///
/// - *Estimator*: raw adjacent-pair counts, no smoothing, so a transition never
///   observed has probability zero and is never generated.
/// - *Initial state*: the observed first mark, held fixed. The alternative is a
///   draw from the marginal, which would add sampling noise to a quantity one
///   observation already fixes; holding it also makes this construction
///   comparable to [`doublet_null_seeded`], which preserves it exactly.
/// - *Dead ends*: a state with no outgoing transition — only the terminal mark
///   can be one — is escaped by drawing the next state from the empirical
///   marginal. Each such draw is counted by [`markov_fallbacks`] and reported.
/// - *Length*: exactly the observed length.
/// - *End of sequence*: not modelled. Generation stops at the observed length,
///   which is what keeps the search space the same size on both paths.
///
/// **Neither mark marginals nor transition counts are preserved exactly by
/// this**, and this module never says they are. [`fidelity`] measures how far
/// they move.
pub fn markov_null_seeded<'a>(sequence: &EventSequence<'a>, seed: u64) -> EventSequence<'a> {
    let (vocabulary, path) = states(sequence);
    let generated = markov_path(&path, vocabulary.len(), seed).0;
    with_marks(sequence, &vocabulary, &generated)
}

/// The state-path half of [`markov_null_seeded`], with the count of dead-end
/// escapes it needed.
pub fn markov_path(path: &[usize], vertices: usize, seed: u64) -> (Vec<usize>, usize) {
    if path.len() < 2 || vertices == 0 {
        return (path.to_vec(), 0);
    }
    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); vertices];
    for pair in path.windows(2) {
        successors[pair[0]].push(pair[1]);
    }
    // The empirical marginal, as a bag of states to draw from uniformly.
    let marginal: Vec<usize> = path.to_vec();

    let mut generator = Lcg::new(seed);
    let mut walked = Vec::with_capacity(path.len());
    let mut fallbacks = 0usize;
    let mut current = path[0];
    walked.push(current);
    for _ in 1..path.len() {
        let choices = &successors[current];
        let next = if choices.is_empty() {
            fallbacks += 1;
            marginal[generator.next_below(marginal.len())]
        } else {
            choices[generator.next_below(choices.len())]
        };
        walked.push(next);
        current = next;
    }
    (walked, fallbacks)
}

/// Dead-end escapes the fitted chain needed for one replicate.
pub fn markov_fallbacks(sequence: &EventSequence<'_>, seed: u64) -> usize {
    let (vocabulary, path) = states(sequence);
    markov_path(&path, vocabulary.len(), seed).1
}

// ---------------------------------------------------------------------------
// Fidelity — did the construction preserve what it was commissioned to preserve
// ---------------------------------------------------------------------------

/// How far one replicate's first-order structure sits from the observed one.
///
/// Every quantity is mechanically derived from marks and order. None reads a
/// prompt, a command, a path, or a payload.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Fidelity {
    /// Total-variation distance between the observed and replicate
    /// transition-frequency matrices: `0.5 * sum |p_obs(x,y) - p_rep(x,y)|`
    /// over adjacent-pair frequencies. Zero when counts are identical.
    pub transition_tv: f64,
    /// The largest per-state outgoing-distribution total-variation distance,
    /// over states with at least one observed outgoing transition. A state the
    /// replicate never leaves scores 1.
    pub max_state_tv: f64,
    /// Distinct observed transitions with zero count in the replicate.
    pub absent_transitions: usize,
    /// Total-variation distance between the observed and replicate mark
    /// marginals.
    pub marginal_tv: f64,
}

/// Adjacent-pair counts, keyed by state index.
fn transitions(path: &[usize]) -> BTreeMap<(usize, usize), usize> {
    let mut counts = BTreeMap::new();
    for pair in path.windows(2) {
        *counts.entry((pair[0], pair[1])).or_insert(0) += 1;
    }
    counts
}

/// Mark counts, keyed by state index.
fn occupancy(path: &[usize], vertices: usize) -> Vec<usize> {
    let mut counts = vec![0usize; vertices];
    for state in path {
        counts[*state] += 1;
    }
    counts
}

/// Compare one replicate's first-order structure with the observed sequence's.
///
/// Both are read through the **observed** sequence's state space, so a replicate
/// that never produces a mark still scores against the full vocabulary.
pub fn fidelity(observed: &EventSequence<'_>, replicate: &EventSequence<'_>) -> Fidelity {
    let (vocabulary, path) = states(observed);
    let mut replicate_path: Vec<usize> = Vec::with_capacity(replicate.events.len());
    for event in &replicate.events {
        match vocabulary.iter().position(|mark| *mark == event.mark) {
            Some(index) => replicate_path.push(index),
            // A construction that invents a mark would be a defect, not a
            // fidelity finding; neither of these can, and the count says so.
            None => return worst_case(),
        }
    }

    let observed_counts = transitions(&path);
    let replicate_counts = transitions(&replicate_path);
    let observed_pairs = path.len().saturating_sub(1) as f64;
    let replicate_pairs = replicate_path.len().saturating_sub(1) as f64;

    let mut keys: Vec<(usize, usize)> = observed_counts.keys().copied().collect();
    for key in replicate_counts.keys() {
        if !keys.contains(key) {
            keys.push(*key);
        }
    }
    let transition_tv = 0.5
        * keys
            .iter()
            .map(|key| {
                let left = *observed_counts.get(key).unwrap_or(&0) as f64 / observed_pairs.max(1.0);
                let right =
                    *replicate_counts.get(key).unwrap_or(&0) as f64 / replicate_pairs.max(1.0);
                (left - right).abs()
            })
            .sum::<f64>();

    let absent_transitions = observed_counts
        .keys()
        .filter(|key| !replicate_counts.contains_key(key))
        .count();

    // Per-state outgoing distributions.
    let mut max_state_tv: f64 = 0.0;
    for state in 0..vocabulary.len() {
        let observed_out: usize = observed_counts
            .iter()
            .filter(|((from, _), _)| *from == state)
            .map(|(_, count)| *count)
            .sum();
        if observed_out == 0 {
            continue;
        }
        let replicate_out: usize = replicate_counts
            .iter()
            .filter(|((from, _), _)| *from == state)
            .map(|(_, count)| *count)
            .sum();
        if replicate_out == 0 {
            max_state_tv = 1.0;
            continue;
        }
        let mut divergence = 0.0;
        for target in 0..vocabulary.len() {
            let left =
                *observed_counts.get(&(state, target)).unwrap_or(&0) as f64 / observed_out as f64;
            let right =
                *replicate_counts.get(&(state, target)).unwrap_or(&0) as f64 / replicate_out as f64;
            divergence += (left - right).abs();
        }
        max_state_tv = max_state_tv.max(0.5 * divergence);
    }

    let observed_occupancy = occupancy(&path, vocabulary.len());
    let replicate_occupancy = occupancy(&replicate_path, vocabulary.len());
    let marginal_tv = 0.5
        * observed_occupancy
            .iter()
            .zip(&replicate_occupancy)
            .map(|(left, right)| {
                (*left as f64 / path.len().max(1) as f64
                    - *right as f64 / replicate_path.len().max(1) as f64)
                    .abs()
            })
            .sum::<f64>();

    Fidelity {
        transition_tv,
        max_state_tv,
        absent_transitions,
        marginal_tv,
    }
}

fn worst_case() -> Fidelity {
    Fidelity {
        transition_tv: 1.0,
        max_state_tv: 1.0,
        absent_transitions: usize::MAX,
        marginal_tv: 1.0,
    }
}

// ---------------------------------------------------------------------------
// Degeneracy — did the construction destroy anything at all
// ---------------------------------------------------------------------------

/// Whether a null can move a given sequence at all, and how far.
///
/// A null that returns its own input cannot separate from anything, and at short
/// recording lengths an exact transition-preserving construction can have very
/// few sequences to choose between. That is a property of the specimen, and it
/// is measured rather than assumed away.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Degeneracy {
    /// Replicates generated.
    pub replicates: usize,
    /// Replicates whose mark sequence equals the observed one exactly.
    pub identical: usize,
    /// Distinct mark sequences among the replicates.
    pub distinct: usize,
    /// `identical / replicates`.
    pub identical_fraction: f64,
}

/// Measure how much a null construction moves one sequence.
pub fn degeneracy<'a, F>(
    sequence: &EventSequence<'a>,
    replicates: usize,
    mut construct: F,
) -> Degeneracy
where
    F: FnMut(&EventSequence<'a>, usize) -> EventSequence<'a>,
{
    let observed: Vec<String> = sequence
        .events
        .iter()
        .map(|event| event.mark.label())
        .collect();
    let mut identical = 0usize;
    let mut seen: Vec<Vec<String>> = Vec::new();
    for index in 0..replicates {
        let replicate = construct(sequence, index);
        let labels: Vec<String> = replicate
            .events
            .iter()
            .map(|event| event.mark.label())
            .collect();
        if labels == observed {
            identical += 1;
        }
        if !seen.contains(&labels) {
            seen.push(labels);
        }
    }
    Degeneracy {
        replicates,
        identical,
        distinct: seen.len(),
        identical_fraction: if replicates == 0 {
            f64::NAN
        } else {
            identical as f64 / replicates as f64
        },
    }
}

// ---------------------------------------------------------------------------
// Controlled fixtures with a first-order background
// ---------------------------------------------------------------------------

/// The transition support of the controls' background chain: from state `i`,
/// the six reachable states `i+1`, `i+2`, `i+4`, `i+5`, `i+7`, `i+9`, modulo
/// twelve.
///
/// Written out rather than computed so that it can be read and checked. Six
/// distinct successors per state and no self-loop, which makes the background's
/// own immediate repetition rate exactly zero — the property every observational
/// specimen turned out to have, arrived at here by a domain-neutral construction
/// rather than by imitating one.
///
/// **Six rather than four, and the reason is recorded rather than hidden.** The
/// first draft branched four ways. At that width the longest run of marks two
/// sequences of these lengths share *by chance* has a median of 8 and a null
/// maximum of 14 — see [`longest_shared_run`] — so a twelve-mark plant would not
/// have cleared what the background produces on its own, and the positive
/// control would have been unable to discriminate whatever the search did. The
/// width was chosen from that diagnostic, before any criterion was preregistered
/// and before `T` was computed at either setting.
pub const SUCCESSORS: [[usize; 6]; 12] = [
    [1, 2, 4, 5, 7, 9],
    [2, 3, 5, 6, 8, 10],
    [3, 4, 6, 7, 9, 11],
    [4, 5, 7, 8, 10, 0],
    [5, 6, 8, 9, 11, 1],
    [6, 7, 9, 10, 0, 2],
    [7, 8, 10, 11, 1, 3],
    [8, 9, 11, 0, 2, 4],
    [9, 10, 0, 1, 3, 5],
    [10, 11, 1, 2, 4, 6],
    [11, 0, 2, 3, 5, 7],
    [0, 1, 3, 4, 6, 8],
];

/// Where the background walk starts.
pub const INITIAL_STATE: usize = 0;

/// The figure the positive control plants: twelve states, matching the ladder's
/// longest span.
///
/// **Every one of its eleven transitions is in [`SUCCESSORS`]**, so planting it
/// adds no transition the background could not have produced. What it adds is a
/// *specific long path, twice per sequence* — structure the first-order
/// transition matrix does not determine, since reproducing it requires eleven
/// particular choices out of four.
pub const FIRST_ORDER_FIGURE: [usize; 12] = [3, 4, 8, 9, 1, 5, 6, 10, 11, 0, 7, 2];

/// Earliest positions at which each control sequence's plants may begin.
pub const PLANT_AFTER_A: [usize; 2] = [20, 100];
/// The same for the shorter sequence.
pub const PLANT_AFTER_B: [usize; 2] = [10, 55];

/// Lengths of the two control sequences, unchanged from sprint:19 so that the
/// search space, and therefore the cost of a complete search, is the same.
pub const CONTROL_LENGTHS: (usize, usize) = (160, 90);

/// Seeds for the two control sequences.
pub const CONTROL_SEEDS: (u64, u64) = (0x000D_0B1E_0000_0001, 0x000D_0B1E_0000_0002);

/// One background walk on the chain: `length` states from [`INITIAL_STATE`],
/// each step drawn from the current state's six successors with probability
/// proportional to [`super::calibration::SYNTHETIC_WEIGHTS`] at the target.
pub fn background_walk(length: usize, seed: u64) -> Vec<usize> {
    let mut generator = Lcg::new(seed);
    let mut path = Vec::with_capacity(length);
    let mut state = INITIAL_STATE;
    for _ in 0..length {
        path.push(state);
        let successors = SUCCESSORS[state];
        let total: u32 = successors
            .iter()
            .map(|target| super::calibration::SYNTHETIC_WEIGHTS[*target])
            .sum();
        let draw = generator.next_below(total as usize) as u32;
        let mut running = 0u32;
        let mut chosen = successors[successors.len() - 1];
        for target in successors {
            running += super::calibration::SYNTHETIC_WEIGHTS[target];
            if draw < running {
                chosen = target;
                break;
            }
        }
        state = chosen;
    }
    path
}

/// Overwrite [`FIRST_ORDER_FIGURE`] into a background walk, at sites chosen so
/// that **no transition outside [`SUCCESSORS`] is created**.
///
/// A plant begins at the first position at or after its site where the walk is
/// already in the figure's first state *and* the state the walk returns to is a
/// legal successor of the figure's last. The entry and exit transitions are then
/// both ones the background could have produced, so the only first-order
/// disturbance planting causes is a shift in transition *frequencies*, which is
/// measured rather than assumed small.
///
/// Returns the positions actually planted.
pub fn plant_figure(path: &mut [usize], sites: &[usize]) -> Vec<usize> {
    let figure = FIRST_ORDER_FIGURE;
    let last = figure[figure.len() - 1];
    let mut planted: Vec<usize> = Vec::new();
    for site in sites {
        let mut position = *site;
        while position + figure.len() < path.len() {
            let clear = planted
                .iter()
                .all(|start| position >= start + figure.len() || position + figure.len() <= *start);
            if clear
                && path[position] == figure[0]
                && SUCCESSORS[last].contains(&path[position + figure.len()])
            {
                path[position..position + figure.len()].copy_from_slice(&figure);
                planted.push(position);
                break;
            }
            position += 1;
        }
    }
    planted
}

/// Build a control [`EventSequence`] from a state path.
///
/// Gaps are drawn from the same fixed distribution sprint:19's controls used, so
/// the timing skeleton of a sprint:20 control is the same kind of object as a
/// sprint:19 control's. Marks are [`super::calibration::SYNTHETIC_TOOLS`], and
/// every one announces itself as synthetic.
pub fn control_sequence<'a>(path: &[usize], seed: u64, session_id: &'a str) -> EventSequence<'a> {
    use super::super::inspection::{EventKind, ExaminedScope, Receipts, RecordCount, V2Kind};
    use super::event_sequence::{ChannelScope, MarkedEvent};

    let mut generator = Lcg::new(seed);
    let mut events = Vec::with_capacity(path.len());
    let mut offset_ms = 0u64;
    for (position, state) in path.iter().enumerate() {
        let gap = 500 + generator.next_below(4_500) as u64;
        if position > 0 {
            offset_ms = offset_ms.saturating_add(gap);
        }
        events.push(MarkedEvent {
            sequence: None,
            mark: Mark {
                kind: EventKind::V2(V2Kind::ToolRequested),
                tool_name: Some(super::calibration::SYNTHETIC_TOOLS[*state]),
            },
            offset_ms,
            gap_from_previous_ms: if position == 0 { None } else { Some(gap) },
        });
    }
    let scope = ExaminedScope::CompleteRecording {
        records: path.len(),
    };
    EventSequence {
        channels: ChannelScope::Observed,
        events,
        origin: jiff::Timestamp::UNIX_EPOCH,
        filtered_out: 0,
        clamped_gaps: 0,
        non_monotonic: RecordCount {
            records: Receipts::default(),
            scope,
        },
        scope,
        session_id: Some(session_id),
    }
}

/// A controlled specimen pair with a first-order background, and where its
/// figure ended up.
pub struct FirstOrderControl<'a> {
    /// The longer sequence.
    pub first: EventSequence<'a>,
    /// The shorter one.
    pub second: EventSequence<'a>,
    /// Positions planted in each, empty for the negative control.
    pub planted: (Vec<usize>, Vec<usize>),
}

/// Build both controls from one pair of background walks.
///
/// The negative and positive controls share the **same** background walk, seed
/// for seed, and differ only at the planted positions. Nothing downstream of a
/// plant is regenerated, so the two fixtures remain comparable event by event.
fn first_order_controls<'a>(plant: bool) -> FirstOrderControl<'a> {
    let (length_a, length_b) = CONTROL_LENGTHS;
    let (seed_a, seed_b) = CONTROL_SEEDS;
    let mut path_a = background_walk(length_a, seed_a);
    let mut path_b = background_walk(length_b, seed_b);
    let planted = if plant {
        (
            plant_figure(&mut path_a, &PLANT_AFTER_A),
            plant_figure(&mut path_b, &PLANT_AFTER_B),
        )
    } else {
        (Vec::new(), Vec::new())
    };
    let (name_a, name_b) = if plant {
        ("fo-posctl-a", "fo-posctl-b")
    } else {
        ("fo-negctl-a", "fo-negctl-b")
    };
    FirstOrderControl {
        first: control_sequence(&path_a, seed_a, name_a),
        second: control_sequence(&path_b, seed_b, name_b),
        planted,
    }
}

/// The negative control: two sequences generated by a **known first-order
/// chain**, with no planted figure beyond what that chain produces naturally.
pub fn first_order_negative<'a>() -> FirstOrderControl<'a> {
    first_order_controls(false)
}

/// The positive control: the same two background walks with
/// [`FIRST_ORDER_FIGURE`] planted twice in each, at sites that create no
/// transition the background could not have produced.
pub fn first_order_positive<'a>() -> FirstOrderControl<'a> {
    first_order_controls(true)
}

/// The length of the longest run of marks appearing contiguously in **both**
/// sequences.
///
/// The cross-sequence half of the destroy side, and the quantity a controlled
/// fixture's planted figure most directly creates: a figure planted in two
/// sequences puts a shared run of its own length into the pair, and a null that
/// destroys longer-range reuse must not reproduce one.
///
/// **A diagnostic, not a detector.** It ranks nothing, selects no candidate,
/// enters no criterion about the corpus, and is never compared with `T`. It
/// reports a length; it reads no mark's identity into any output.
pub fn longest_shared_run(first: &EventSequence<'_>, second: &EventSequence<'_>) -> usize {
    let left: Vec<Mark<'_>> = first.events.iter().map(|event| event.mark).collect();
    let right: Vec<Mark<'_>> = second.events.iter().map(|event| event.mark).collect();
    if left.is_empty() || right.is_empty() {
        return 0;
    }
    let mut previous = vec![0usize; right.len() + 1];
    let mut current = vec![0usize; right.len() + 1];
    let mut longest = 0usize;
    for mark in &left {
        for (b, other) in right.iter().enumerate() {
            current[b + 1] = if mark == other { previous[b] + 1 } else { 0 };
            longest = longest.max(current[b + 1]);
        }
        std::mem::swap(&mut previous, &mut current);
        current.iter_mut().for_each(|cell| *cell = 0);
    }
    longest
}

/// How many distinct marks a sequence delivered.
///
/// A count, publishable under decision:8, and the quantity that governs how much
/// freedom an exact transition-preserving null has: a vocabulary approaching the
/// event count leaves a transition graph with almost nothing to permute.
pub fn vocabulary_size(sequence: &EventSequence<'_>) -> usize {
    states(sequence).0.len()
}

/// Occurrences of `n`-grams that occur more than once in the sequence.
///
/// The destroy side of the preserve/destroy table, and descriptive only: no
/// criterion in this round reads it. It counts *positions*, so a figure of
/// length `n` appearing three times contributes three.
pub fn repeated_ngrams(sequence: &EventSequence<'_>, n: usize) -> usize {
    let (_, path) = states(sequence);
    if n == 0 || path.len() < n {
        return 0;
    }
    let mut counts: BTreeMap<Vec<usize>, usize> = BTreeMap::new();
    for window in path.windows(n) {
        *counts.entry(window.to_vec()).or_insert(0) += 1;
    }
    counts.values().filter(|count| **count > 1).sum::<usize>()
}
