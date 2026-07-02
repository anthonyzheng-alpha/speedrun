// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Shared "honesty envelope" math used by the memory model and the
//! performance/readiness metrics: a Wilson score interval whose width shrinks
//! as more data accumulates, plus a confidence derived from that width.

/// z-score for the Wilson interval (roughly a 95% confidence level).
pub(crate) const WILSON_Z: f32 = 1.96;

/// Wilson score interval for a binomial proportion `p` observed over an
/// effective sample size `n`. Returns the `(lower, upper)` bounds clamped to
/// `0.0..=1.0`. The interval always contains `p` and narrows as `n` grows.
///
/// `n` is an `f32` so callers can pass a *weighted* effective sample size
/// (e.g. practice-exam questions counted more heavily than flashcard reviews).
pub(crate) fn wilson_interval(p: f32, n: f32) -> (f32, f32) {
    if n <= 0.0 {
        return (0.0, 1.0);
    }
    let z = WILSON_Z;
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let margin = (z / denom) * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
    (
        (center - margin).clamp(0.0, 1.0),
        (center + margin).clamp(0.0, 1.0),
    )
}

/// Confidence percentage (0-100) derived from a range expressed on the 0-100
/// scale. A tighter range means we're more confident.
pub(crate) fn confidence_from_range(range_min: f32, range_max: f32) -> f32 {
    (100.0 * (1.0 - (range_max - range_min) / 100.0)).clamp(0.0, 100.0)
}
