// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The memory model.
//!
//! Estimates the chance (0-100) that the user recalls the fact taught on a
//! card. The point estimate reuses the FSRS retrievability that Anki already
//! computes (which accounts for time since the last review); on top of that we
//! report an honesty envelope:
//!
//!   - a plausible range, computed as a Wilson score interval whose width
//!     shrinks as the card accumulates more reviews,
//!   - a confidence percentage derived from that range's width, and
//!   - a short justification.
//!
//! When there isn't enough review history to state a number confidently (or the
//! card has no FSRS memory state yet), we report `has_enough_data = false` and a
//! justification saying so, rather than inventing a figure.

use anki_proto::stats::CardMemoryEstimate;

use super::envelope::confidence_from_range;
use super::envelope::wilson_interval;

/// Minimum number of rated reviews required before we're willing to state a
/// number. Below this there isn't enough signal, so we say so instead.
const MIN_MEMORY_SAMPLES: u32 = 3;

/// Build a [`CardMemoryEstimate`] from the FSRS point estimate and the card's
/// review history.
///
/// * `retrievability` is the FSRS recall probability in `0.0..=1.0`, or `None`
///   when the card has no memory state yet.
/// * `n_reviews` is the number of rated reviews the card has had.
/// * `successes` is how many of those reviews were recalled (not `Again`).
/// * `last_review_secs` is the Unix timestamp (seconds) of the last review.
pub(crate) fn memory_estimate(
    retrievability: Option<f32>,
    n_reviews: u32,
    successes: u32,
    last_review_secs: i64,
) -> CardMemoryEstimate {
    let Some(retrievability) = retrievability else {
        return insufficient_data(n_reviews, last_review_secs);
    };
    if n_reviews < MIN_MEMORY_SAMPLES {
        return insufficient_data(n_reviews, last_review_secs);
    }

    let p = retrievability.clamp(0.0, 1.0);
    let (lo, hi) = wilson_interval(p, n_reviews as f32);
    let range_min = lo * 100.0;
    let range_max = hi * 100.0;
    let score = p * 100.0;
    let confidence = confidence_from_range(range_min, range_max);

    let success_rate = (successes as f32 / n_reviews as f32 * 100.0).round() as i32;
    let justification = format!(
        "Based on {n_reviews} reviews ({success_rate}% recalled so far), \
         adjusted for the time since your last review.",
    );

    CardMemoryEstimate {
        has_enough_data: true,
        score,
        range_min,
        range_max,
        confidence,
        last_updated: last_review_secs,
        justification,
    }
}

/// The response used whenever we can't confidently state a number.
fn insufficient_data(n_reviews: u32, last_review_secs: i64) -> CardMemoryEstimate {
    let justification = if n_reviews == 0 {
        "Not enough data yet: this card hasn't been reviewed. \
         Keep studying it to get a recall estimate."
            .to_string()
    } else {
        format!(
            "Not enough data yet: only {n_reviews} review(s) so far. \
             Keep studying this card to get a reliable recall estimate.",
        )
    };
    CardMemoryEstimate {
        has_enough_data: false,
        score: 0.0,
        range_min: 0.0,
        range_max: 0.0,
        confidence: 0.0,
        last_updated: last_review_secs,
        justification,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insufficient_data_reports_no_number() {
        // No FSRS memory state -> no number, regardless of review count.
        let no_state = memory_estimate(None, 50, 45, 0);
        assert!(!no_state.has_enough_data);
        assert!(!no_state.justification.is_empty());

        // Too few reviews -> no number, even with a memory state.
        let too_few = memory_estimate(Some(0.9), MIN_MEMORY_SAMPLES - 1, 1, 0);
        assert!(!too_few.has_enough_data);
        assert!(!too_few.justification.is_empty());

        // Exactly at the threshold -> we're willing to state a number.
        let enough = memory_estimate(Some(0.9), MIN_MEMORY_SAMPLES, 3, 0);
        assert!(enough.has_enough_data);
    }

    #[test]
    fn interval_narrows_with_more_reviews() {
        let few = memory_estimate(Some(0.8), 5, 4, 0);
        let many = memory_estimate(Some(0.8), 200, 160, 0);

        let few_width = few.range_max - few.range_min;
        let many_width = many.range_max - many.range_min;
        assert!(
            many_width < few_width,
            "more reviews should tighten the range ({many_width} < {few_width})"
        );
        assert!(
            many.confidence > few.confidence,
            "more reviews should raise confidence ({} > {})",
            many.confidence,
            few.confidence
        );
    }

    #[test]
    fn score_matches_retrievability_and_within_range() {
        let est = memory_estimate(Some(0.75), 20, 16, 0);
        assert!((est.score - 75.0).abs() < 1e-3);
        assert!(
            est.range_min <= est.score && est.score <= est.range_max,
            "score {} should lie within [{}, {}]",
            est.score,
            est.range_min,
            est.range_max
        );
    }

    #[test]
    fn bounds_are_clamped() {
        for &p in &[0.0, 0.5, 1.0] {
            let est = memory_estimate(Some(p), 10, (p * 10.0) as u32, 0);
            assert!(
                (0.0..=100.0).contains(&est.range_min),
                "range_min out of bounds: {}",
                est.range_min
            );
            assert!(
                (0.0..=100.0).contains(&est.range_max),
                "range_max out of bounds: {}",
                est.range_max
            );
            assert!(est.range_min <= est.range_max);
            assert!((0.0..=100.0).contains(&est.confidence));
        }
        // Out-of-range retrievability is clamped before use.
        let high = memory_estimate(Some(2.0), 10, 10, 0);
        assert!((high.score - 100.0).abs() < 1e-3);
        assert!(high.range_max <= 100.0);
    }
}
