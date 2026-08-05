//! sprint:14's representation audit.
//!
//! **Disposable.** Deleted with the audit.
//!
//! **What these tests are for.** The audit's claims are structural, so they are
//! testable as structure rather than as measurement: that the representation is
//! closed over what a scorer may see, that Family E's arms differ inside it,
//! that the witness pair does not, and that every preregistered function is
//! invariant under relabelling the mark alphabet — which is the property that
//! makes it a function of the representation rather than of the data behind it.

use std::collections::BTreeMap;

use witnessglass::experiment::identifiability::{Observation, SCORERS, witness};

fn labels(marks: &[&str]) -> Vec<String> {
    marks.iter().map(|mark| (*mark).to_owned()).collect()
}

fn population(pairs: &[(&str, usize)]) -> BTreeMap<String, usize> {
    pairs
        .iter()
        .map(|(mark, count)| ((*mark).to_owned(), *count))
        .collect()
}

fn observation(a: &[&str], b: &[&str], counts: &[(&str, usize)], total: usize) -> Observation {
    Observation {
        a: labels(a),
        b: labels(b),
        a_counts: population(counts),
        b_counts: population(counts),
        a_total: total,
        b_total: total,
    }
}

/// Family E's two arms, exactly as sprint:12 builds them.
fn family_e_arms() -> (Observation, Observation) {
    (
        observation(
            &["x", "y", "z", "w"],
            &["x", "y", "z", "w"],
            &[("x", 1), ("y", 1), ("z", 1), ("w", 1), ("bg", 40)],
            44,
        ),
        observation(
            &["x", "y", "z", "x"],
            &["x", "y", "z", "x"],
            &[("x", 2), ("y", 1), ("z", 1), ("bg", 40)],
            44,
        ),
    )
}

#[test]
fn family_es_arms_differ_inside_the_representation() {
    // task:24 §A.2, as an assertion rather than a paragraph. The arms are not
    // R-identical, so no collision certificate exists for them and the
    // distinction is identifiable in principle.
    let (novel, redundant) = family_e_arms();
    assert_ne!(novel.equality_pattern(), redundant.equality_pattern());
    assert_eq!(novel.tail_repeats_span(), Some(false));
    assert_eq!(redundant.tail_repeats_span(), Some(true));
}

#[test]
fn the_witness_pair_is_identical_inside_the_representation() {
    // task:24 §A.3. Two candidates a scorer cannot tell apart, whose desired
    // orderings are opposite — so *semantic* redundancy is not a function of the
    // representation, whatever Family E's syntactic arms do.
    let (p, q) = witness::pair();
    assert_eq!(p, q, "the witness arms must be indistinguishable");
    assert_eq!(p.equality_pattern(), q.equality_pattern());
    // And every preregistered function agrees with them, necessarily.
    for scorer in SCORERS.iter() {
        assert_eq!(
            (scorer.score)(&p),
            (scorer.score)(&q),
            "{} separated two identical observations",
            scorer.name
        );
    }
}

#[test]
fn every_function_is_invariant_under_relabelling_the_mark_alphabet() {
    // The property that makes each one a function of the representation. A mark
    // is an opaque label; renaming every mark consistently must change nothing.
    let cases = [
        family_e_arms().0,
        family_e_arms().1,
        observation(
            &["p", "q", "p", "r"],
            &["p", "q", "s", "r"],
            &[("p", 3), ("q", 1), ("r", 2), ("s", 5), ("bg", 20)],
            31,
        ),
    ];

    let rename = |mark: &str| format!("zz-{mark}-zz");
    for original in cases {
        let renamed = Observation {
            a: original.a.iter().map(|m| rename(m)).collect(),
            b: original.b.iter().map(|m| rename(m)).collect(),
            a_counts: original
                .a_counts
                .iter()
                .map(|(m, c)| (rename(m), *c))
                .collect(),
            b_counts: original
                .b_counts
                .iter()
                .map(|(m, c)| (rename(m), *c))
                .collect(),
            a_total: original.a_total,
            b_total: original.b_total,
        };
        for scorer in SCORERS.iter() {
            let before = (scorer.score)(&original);
            let after = (scorer.score)(&renamed);
            match (before, after) {
                (Some(left), Some(right)) => assert!(
                    (left - right).abs() < 1e-12,
                    "{} moved under relabelling: {left} against {right}",
                    scorer.name
                ),
                (None, None) => {}
                _ => panic!("{} changed definedness under relabelling", scorer.name),
            }
        }
    }
}

#[test]
fn the_preregistered_family_is_the_one_task_24_fixed() {
    // Ten functions, in order, with the two probes marked. If a later round adds
    // or reorders one, the enumeration stops being the preregistered one.
    let names: Vec<&str> = SCORERS.iter().map(|scorer| scorer.name).collect();
    assert_eq!(
        names,
        vec![
            "agreements",
            "agreement_rate",
            "distinct_agreements",
            "distinct_agreement_rate",
            "span_distinct",
            "first_occurrence_agreements",
            "negative_repeats",
            "surprisal",
            "rarity_of_agreements",
            "novel_rarity",
        ]
    );
    let probes: Vec<&str> = SCORERS
        .iter()
        .filter(|scorer| scorer.probe)
        .map(|scorer| scorer.name)
        .collect();
    assert_eq!(probes, vec!["rarity_of_agreements", "novel_rarity"]);
}

#[test]
fn every_function_declines_across_an_indel_rather_than_guessing() {
    let uneven = Observation {
        a: labels(&["x", "y", "z"]),
        b: labels(&["x", "y", "z", "w"]),
        a_counts: population(&[("x", 1), ("y", 1), ("z", 1), ("bg", 10)]),
        b_counts: population(&[("x", 1), ("y", 1), ("z", 1), ("w", 1), ("bg", 10)]),
        a_total: 13,
        b_total: 14,
    };
    assert_eq!(uneven.len(), None);
    for scorer in SCORERS.iter() {
        assert_eq!(
            (scorer.score)(&uneven),
            None,
            "{} scored a candidate with no positional correspondence",
            scorer.name
        );
    }
}

#[test]
fn the_representation_separates_family_es_arms_under_six_of_the_ten_functions() {
    // The empirical counterpart of §A.2: the information is not merely present,
    // it is exposed by ordinary functions. Recorded as a count rather than a
    // list so that adding a function later cannot silently change the claim.
    let (novel, redundant) = family_e_arms();
    let separating = SCORERS
        .iter()
        .filter(
            |scorer| match ((scorer.score)(&novel), (scorer.score)(&redundant)) {
                (Some(left), Some(right)) => (left - right).abs() > 1e-12,
                _ => false,
            },
        )
        .count();
    assert!(
        separating >= 6,
        "only {separating} of ten functions separate the arms"
    );
}
