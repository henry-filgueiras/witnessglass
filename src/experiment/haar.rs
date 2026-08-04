//! A Haar multiresolution decomposition of one scalar behavioral dimension.
//!
//! **Disposable.** See [`crate::experiment`]. sprint:5, task:15.
//!
//! This is the smallest transform that answers the question sprint:5 asks, and
//! deliberately not a wavelet library. There is one wavelet, no inverse, no
//! filter bank, no boundary-extension modes, and no framework. If a second
//! wavelet is ever wanted, the right move is to read this file and decide
//! whether it was worth keeping at all.
//!
//! # What it does not do
//!
//! **It does not choose the sampling interval, and it cannot see below it.** The
//! input is an already-sampled signal — [`crate::experiment::signal`]'s buckets,
//! 500 ms wide by default. Structure faster than that interval is absent from
//! the input, which is a different statement from being absent from the session.
//! Nothing here derives, justifies, or optimises a bucket width; sprint:4 chose
//! one and this operates on the result.
//!
//! What it does offer is a decomposition across dyadic scales *representable
//! from that sampling*, from which the distribution of energy is evidence about
//! which scales carry behaviour. That evidence can inform a later choice of
//! subsequence length or of a coarser aggregation. It is not a derivation of an
//! optimal one.
//!
//! # Convention
//!
//! Orthonormal Haar. For each adjacent pair of the current approximation:
//!
//! ```text
//! a[k] = (x[2k] + x[2k+1]) / √2
//! d[k] = (x[2k] − x[2k+1]) / √2
//! ```
//!
//! applied repeatedly to the approximation until fewer than two samples remain.
//! The `1/√2` scaling is what makes the transform orthonormal, and orthonormality
//! is why [`Decomposition::energy_identity_residual`] can be an exact test rather
//! than a plausible claim: the sum of squares is preserved at every level.
//!
//! A level-`L` detail coefficient is computed from `2^L` consecutive base samples
//! and measures the contrast between two adjacent means, each `2^(L−1)` samples
//! wide. Two scales could be reported for that and they differ by a factor of
//! two, so this module fixes one: [`Level::scale_ms`] is the **window**,
//! `2^L × base_ms`. At a 500 ms base that reads level 1 = 1 s, 2 = 2 s, 3 = 4 s,
//! 4 = 8 s, 5 = 16 s, and upward. [`Level::contrast_ms`] carries the half-window
//! beside it, so neither reading has to be inferred.
//!
//! # Inputs whose length is not a power of two
//!
//! Real recordings do not end on a power of two and never will. The policy here
//! is chosen so that nothing is invented and nothing is dropped:
//!
//! **At each level, if the current approximation has odd length, its final
//! unpaired sample is set aside as a [`Remainder`] — recorded with its value, its
//! index, and its energy — and does not propagate to the next level.**
//!
//! The alternatives were considered and rejected. Zero-padding to the next power
//! of two invents values, and in *this* signal a zero is a meaningful
//! observation — "no record in that interval" — so padding would be fabricating
//! evidence of quiet. Periodic or symmetric extension invents values too, and
//! worse, invents ones that look like the data. Truncating to a power of two
//! discards up to half the recording. Carrying an unpaired sample forward
//! unscaled breaks orthonormality and with it the only exact check this module
//! has.
//!
//! Setting the sample aside costs one thing and it is worth naming precisely,
//! because it is sharper than it first looks. A remainder is excluded from the
//! level that set it aside **and from every coarser level**, since it never
//! reaches the next approximation. A remainder taken at level 1 is therefore
//! represented at *no* level at all.
//!
//! That is not hypothetical. An odd-length signal puts its final base sample in
//! the level-1 remainder, and a dimension whose only activity is in that last
//! bucket — a `session_ended` record at the end of a recording, for instance —
//! decomposes to zero detail energy everywhere. It is a real observation that
//! this transform cannot place at any scale.
//!
//! The remainder list is how a reader sees that rather than having to deduce it,
//! [`Decomposition::silence`] distinguishes it from a genuinely flat dimension,
//! and the energy identity keeps it honest: the remainders' energy is in the
//! balance, so it cannot be quietly lost.
//!
//! # Reading the output
//!
//! [`Decomposition::spectrum`] reports, per level, the detail energy, its share of
//! the total detail energy, and the share an **isolated unit impulse** would
//! produce at that level. That last column is not decoration. A sparse behavioral
//! signal is mostly a train of isolated impulses, and an isolated impulse has
//! detail energy exactly `2^-L` at level `L` — halving at every coarser scale,
//! regardless of what produced it. A spectrum that merely reproduces that decay
//! has measured the sparsity of the recording and not anything about the session.
//! [`Band::ratio_to_impulse_null`] is the departure from it, and it is the number
//! worth looking at.
//!
//! # Two invariances worth knowing before reading any result
//!
//! Detail coefficients are **exactly invariant to a constant offset** — the
//! difference of two values does not move when both move — and scale **linearly
//! with a constant factor**. So energy *shares* are invariant to both. A z-score
//! is exactly an offset and a factor, which means sprint:4's normalization policy
//! cannot change any spectrum in this module, and a dimension's magnitude cannot
//! reach another dimension's spectrum. Both are asserted by tests rather than
//! left as reasoning.

use serde::Serialize;

/// The orthonormal Haar scaling factor.
const SQRT_2: f64 = std::f64::consts::SQRT_2;

/// A sample set aside because the level it belonged to had odd length.
///
/// Not discarded, not padded against, and not propagated. It carries its own
/// energy into [`Decomposition::remainder_energy`] so the energy identity stays
/// exact.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Remainder {
    /// Level at which the sample was unpaired. Level 1 operates on the input.
    pub level: u32,
    /// Index within that level's input sequence.
    pub index: usize,
    /// The value, as it stood at that level.
    pub value: f64,
    /// Its squared value.
    pub energy: f64,
}

/// One dyadic level of the decomposition.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Level {
    /// Level number, from 1. Level 1 is the finest.
    pub level: u32,
    /// Width of the window one coefficient is computed from: `2^level × base_ms`.
    /// This is the scale this module reports.
    pub scale_ms: u64,
    /// Width of each of the two means being contrasted: `2^(level−1) × base_ms`.
    /// Carried so the other reading of "scale" does not have to be inferred.
    pub contrast_ms: u64,
    /// Base samples this level still represents: `detail.len() × 2^level`.
    ///
    /// Below the input length whenever an odd tail was set aside at this level
    /// or any finer one. It falls as levels get coarser, and it is how much of
    /// the recording a coarse-scale reading is actually about — a number worth
    /// seeing before trusting one.
    pub covered_samples: usize,
    /// Detail coefficients at this level, in order.
    pub detail: Vec<f64>,
    /// Sum of squared detail coefficients at this level.
    pub energy: f64,
    /// The sample set aside at this level, if its input had odd length.
    pub remainder: Option<Remainder>,
}

/// A Haar decomposition of one scalar series.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Decomposition {
    /// Sampling interval of the input, in milliseconds. A modeling choice made
    /// upstream, carried here so every scale can be stated in real units and so
    /// no reader mistakes it for something this module derived.
    pub base_ms: u64,
    /// Length of the input series.
    pub input_len: usize,
    /// Sum of squares of the input.
    pub input_energy: f64,
    /// Whether every input sample holds the same value. Kept because it is the
    /// only way to tell a genuinely flat dimension from one whose variation the
    /// odd-length policy set aside — see [`Decomposition::silence`].
    pub input_is_constant: bool,
    /// Levels, finest first.
    pub levels: Vec<Level>,
    /// The coarsest approximation, left when fewer than two samples remain.
    pub approximation: Vec<f64>,
    /// Sum of squares of the coarsest approximation.
    pub approximation_energy: f64,
    /// Sum of squares of every detail coefficient at every level.
    pub detail_energy: f64,
    /// Sum of squares of every sample set aside as a remainder.
    pub remainder_energy: f64,
}

impl Decomposition {
    /// How many levels the input supported.
    pub fn levels(&self) -> usize {
        self.levels.len()
    }

    /// The residual of the exact energy identity:
    ///
    /// ```text
    /// input_energy − (detail_energy + approximation_energy + remainder_energy)
    /// ```
    ///
    /// Zero up to floating-point accumulation, by orthonormality. A test asserts
    /// it rather than this comment.
    pub fn energy_identity_residual(&self) -> f64 {
        self.input_energy - (self.detail_energy + self.approximation_energy + self.remainder_energy)
    }

    /// Why a series has no detail energy, when it has none.
    ///
    /// Zero detail energy has three quite different causes and they license
    /// different readings, so they are not collapsed into one. `None` means the
    /// series does have detail energy and the question does not arise.
    pub fn silence(&self) -> Option<Silence> {
        if self.detail_energy > 0.0 {
            return None;
        }
        Some(if self.input_energy == 0.0 {
            Silence::Empty
        } else if self.input_is_constant {
            Silence::Constant
        } else {
            // A series with no remainders and no detail energy at any level is
            // constant, by induction on the levels. So a series that varies and
            // still produced no detail can only have had its variation set
            // aside by the odd-length policy.
            Silence::OnlyInRemainders
        })
    }

    /// Every remainder, finest level first.
    pub fn remainders(&self) -> Vec<Remainder> {
        self.levels
            .iter()
            .filter_map(|level| level.remainder)
            .collect()
    }

    /// Per-level energy, its share, and the isolated-impulse null it is read
    /// against.
    ///
    /// Empty for an input with no levels. For a series with no detail energy at
    /// all — a constant series, which includes a dimension that is zero
    /// everywhere — shares are zero rather than `NaN`, and
    /// [`Band::ratio_to_impulse_null`] is zero, which reads as "no variation at
    /// any scale" rather than as a suspiciously flat spectrum.
    pub fn spectrum(&self) -> Vec<Band> {
        let levels = self.levels.len() as u32;
        // The null is normalized over the levels this input actually supported,
        // so it sums to 1 against the same denominator the observed shares use.
        let null_total: f64 = (1..=levels).map(|level| 2f64.powi(-(level as i32))).sum();

        self.levels
            .iter()
            .map(|level| {
                let share = if self.detail_energy > 0.0 {
                    level.energy / self.detail_energy
                } else {
                    0.0
                };
                let null = if null_total > 0.0 {
                    2f64.powi(-(level.level as i32)) / null_total
                } else {
                    0.0
                };
                Band {
                    level: level.level,
                    scale_ms: level.scale_ms,
                    energy: level.energy,
                    share,
                    impulse_null_share: null,
                    ratio_to_impulse_null: if null > 0.0 { share / null } else { 0.0 },
                }
            })
            .collect()
    }
}

/// Why a series produced no detail energy.
///
/// Distinguished because "this dimension never varies" and "everything this
/// dimension observed was set aside by the odd-length policy" are different
/// facts, and a reader who cannot tell them apart will read an artefact of the
/// transform as a property of the recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Silence {
    /// The series is zero everywhere. Nothing was observed to vary.
    Empty,
    /// The series is non-zero but never changes. Genuinely flat.
    Constant,
    /// Every non-zero sample was set aside as a remainder and reached no level.
    /// This is the transform's limitation showing, not the recording's.
    OnlyInRemainders,
}

/// One row of a scale spectrum.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Band {
    /// Level number, from 1.
    pub level: u32,
    /// Window width in milliseconds, `2^level × base_ms`.
    pub scale_ms: u64,
    /// Detail energy at this level.
    pub energy: f64,
    /// This level's share of the series' total detail energy.
    pub share: f64,
    /// The share an isolated unit impulse would produce here.
    pub impulse_null_share: f64,
    /// `share / impulse_null_share`. Above 1 is energy in excess of what pure
    /// sparsity explains; below 1 is a deficit. Around 1 says the level is
    /// indistinguishable from isolated impulses, which is what a mostly-empty
    /// recording produces on its own.
    pub ratio_to_impulse_null: f64,
}

/// Decompose one scalar series.
///
/// Pure and deterministic. `base_ms` is carried through for reporting and does
/// not affect a single coefficient.
///
/// An input shorter than two samples yields no levels: there is no pair to
/// contrast, so there is no scale to report, and a zero-level decomposition says
/// that rather than inventing one.
pub fn decompose(samples: &[f64], base_ms: u64) -> Decomposition {
    let input_energy = samples.iter().map(|value| value * value).sum();
    let input_is_constant = samples
        .first()
        .is_none_or(|first| samples.iter().all(|value| value == first));

    let mut current = samples.to_vec();
    let mut levels: Vec<Level> = Vec::new();
    let mut level = 0u32;

    while current.len() >= 2 {
        level += 1;
        let pairs = current.len() / 2;
        let mut approximation = Vec::with_capacity(pairs);
        let mut detail = Vec::with_capacity(pairs);
        for k in 0..pairs {
            let (first, second) = (current[2 * k], current[2 * k + 1]);
            approximation.push((first + second) / SQRT_2);
            detail.push((first - second) / SQRT_2);
        }

        // An odd tail is set aside here and does not reach the next level. It is
        // recorded rather than dropped, and its energy stays in the balance.
        let remainder = (current.len() % 2 == 1).then(|| {
            let index = current.len() - 1;
            let value = current[index];
            Remainder {
                level,
                index,
                value,
                energy: value * value,
            }
        });

        levels.push(Level {
            level,
            scale_ms: base_ms.saturating_mul(1u64.checked_shl(level).unwrap_or(u64::MAX)),
            contrast_ms: base_ms.saturating_mul(1u64.checked_shl(level - 1).unwrap_or(u64::MAX)),
            covered_samples: detail
                .len()
                .saturating_mul(1usize.checked_shl(level).unwrap_or(usize::MAX)),
            energy: detail.iter().map(|value| value * value).sum(),
            detail,
            remainder,
        });

        current = approximation;
    }

    let detail_energy = levels.iter().map(|level| level.energy).sum();
    let remainder_energy = levels
        .iter()
        .filter_map(|level| level.remainder)
        .map(|remainder| remainder.energy)
        .sum();
    let approximation_energy = current.iter().map(|value| value * value).sum();

    Decomposition {
        base_ms,
        input_len: samples.len(),
        input_energy,
        input_is_constant,
        levels,
        approximation: current,
        approximation_energy,
        detail_energy,
        remainder_energy,
    }
}
