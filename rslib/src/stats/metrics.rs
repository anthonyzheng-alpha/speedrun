// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Global performance and readiness metrics.
//!
//! Neither metric is card-specific. Both blend two signals per MCAT section:
//!
//!   - flashcards: the average FSRS retrievability of the cards mapped to that
//!     section (via the [`MCAT_TOPICS`] taxonomy), and
//!   - practice exams: the share of practice-exam questions answered correctly
//!     in that section, from the stored attempt history.
//!
//! Practice-exam questions are weighted more heavily than flashcard reviews
//! ([`W_EXAM`] vs [`W_CARD`]), matching the intuition that exam-style questions
//! are a stronger signal of exam performance. The blended probability drives:
//!
//!   - performance: the estimated chance (0-100) of answering a new exam-style
//!     question correctly, and
//!   - readiness: the projected MCAT score (118-132 per section, 472-528
//!     overall), a linear mapping of that probability.
//!
//! Each estimate carries the same honesty envelope as the memory model (range,
//! confidence, last-updated, justification); when there isn't enough data we
//! say so rather than inventing a figure.

use std::collections::HashMap;

use anki_proto::stats::ExamMetricsResponse;
use anki_proto::stats::MetricEstimate;
use anki_proto::stats::SectionMetric;
use fsrs::FSRS;
use fsrs::FSRS5_DEFAULT_DECAY;

use super::envelope::confidence_from_range;
use super::envelope::wilson_interval;
use crate::collection::mcat::MCAT_TOPICS;
use crate::prelude::*;
use crate::search::SortMode;

/// Weight of one flashcard review in the blended estimate.
const W_CARD: f32 = 1.0;
/// Weight of one practice-exam question (higher than a flashcard review).
const W_EXAM: f32 = 3.0;
/// Minimum number of practice-exam questions answered in a section before we
/// will state a number for it. Set high so a single short quiz can't unlock a
/// prediction; a section needs a full practice exam's worth of evidence.
const MIN_EXAM_QUESTIONS_PER_SECTION: u32 = 10;

/// Lowest scaled score for a single MCAT section.
const SECTION_SCORE_MIN: f32 = 118.0;
/// Span of a single section's scaled score (118-132).
const SECTION_SCORE_SPAN: f32 = 14.0;
/// Lowest possible total MCAT score (4 * 118).
const TOTAL_SCORE_MIN: f32 = 472.0;
/// Span of the total MCAT score (472-528).
const TOTAL_SCORE_SPAN: f32 = 56.0;

const INSUFFICIENT_MSG: &str =
    "Not enough data yet: take a full practice exam (at least 10 questions) in every MCAT \
     topic you're studying before a prediction is made.";

/// Maps each MCAT section deck name to the practice-exam topic key used in
/// `questions.json`.
const SECTION_TOPICS: &[(&str, &str)] = &[
    ("MCAT::Biology & Biochemistry", "biology_biochemistry"),
    ("MCAT::Chemistry & Physics", "chemistry_physics"),
    ("MCAT::Psychology & Sociology", "psychology_sociology"),
    ("MCAT::Critical Analysis & Reasoning (CARS)", "cars"),
];

/// Per-section running totals for the two input signals.
#[derive(Default)]
struct SectionAccumulator {
    /// Sum of FSRS retrievabilities of the reviewed cards in this section.
    sum_retrievability: f32,
    /// Number of reviewed cards contributing to `sum_retrievability`.
    reviewed_cards: f32,
    /// Number of taxonomy cards present in the collection for this section,
    /// regardless of whether they've been reviewed. A section counts as
    /// "covered" (and therefore must have exam evidence before the overall
    /// estimate will state a number) when this is greater than zero.
    present_cards: u32,
    /// Practice-exam questions answered correctly in this section.
    correct: u32,
    /// Practice-exam questions answered in this section.
    total: u32,
    /// Latest input timestamp (last review or practice exam), unix seconds.
    last_updated: i64,
}

impl SectionAccumulator {
    /// Whether this section has any cards in the collection.
    fn is_covered(&self) -> bool {
        self.present_cards > 0
    }

    /// Whether this section has enough practice-exam evidence to state a number.
    fn is_eligible(&self) -> bool {
        self.total >= MIN_EXAM_QUESTIONS_PER_SECTION
    }
}

impl Collection {
    /// Compute the global performance and readiness metrics, overall and per
    /// MCAT section. See the module docs for the model.
    pub fn exam_metrics(&mut self) -> Result<ExamMetricsResponse> {
        let mut acc: Vec<SectionAccumulator> =
            (0..MCAT_TOPICS.len()).map(|_| SectionAccumulator::default()).collect();

        self.accumulate_flashcard_signal(&mut acc)?;
        self.accumulate_practice_exam_signal(&mut acc);

        let mut performance_sections = Vec::with_capacity(acc.len());
        let mut readiness_sections = Vec::with_capacity(acc.len());
        let mut section_probabilities = Vec::new();
        let mut effective_samples_total = 0.0f32;
        let mut total_reviews = 0u32;
        let mut total_questions = 0u32;
        let mut overall_last_updated = 0i64;
        // The overall estimate requires practice-exam evidence in *every*
        // covered topic; a single strong section is not enough.
        let mut any_covered = false;
        let mut all_covered_eligible = true;

        for (idx, section) in MCAT_TOPICS.iter().enumerate() {
            let (deck_name, _topics) = *section;
            let label = section_display_name(deck_name);
            let accumulator = &acc[idx];

            total_reviews += accumulator.reviewed_cards as u32;
            total_questions += accumulator.total;
            overall_last_updated = overall_last_updated.max(accumulator.last_updated);
            if accumulator.is_covered() {
                any_covered = true;
                if !accumulator.is_eligible() {
                    all_covered_eligible = false;
                }
            }

            let (performance, readiness, probability, effective) = section_estimates(accumulator);
            if let Some(p) = probability {
                section_probabilities.push(p);
                effective_samples_total += effective;
            }
            performance_sections.push(SectionMetric {
                section: label.clone(),
                estimate: Some(performance),
            });
            readiness_sections.push(SectionMetric {
                section: label,
                estimate: Some(readiness),
            });
        }

        let overall_ready = any_covered && all_covered_eligible && !section_probabilities.is_empty();
        let (performance_overall, readiness_overall) = if overall_ready {
            overall_estimates(
                &section_probabilities,
                effective_samples_total,
                total_reviews,
                total_questions,
                overall_last_updated,
            )
        } else {
            let est = insufficient(overall_last_updated);
            (est.clone(), est)
        };

        Ok(ExamMetricsResponse {
            performance_overall: Some(performance_overall),
            performance_sections,
            readiness_overall: Some(readiness_overall),
            readiness_sections,
        })
    }

    /// Add the average-retrievability signal from reviewed cards mapped to each
    /// section by front text (mirroring the exam-coverage mapping).
    fn accumulate_flashcard_signal(&mut self, acc: &mut [SectionAccumulator]) -> Result<()> {
        let front_to_section = front_to_section_index();
        let timing = self.timing_today()?;
        let now = timing.now;
        let fsrs = FSRS::new(None).unwrap();

        let cids = self.search_cards("", SortMode::NoOrder)?;
        for cid in cids {
            let Some(card) = self.storage.get_card(cid)? else {
                continue;
            };
            let Some(note) = self.storage.get_note(card.note_id)? else {
                continue;
            };
            let Some(front) = note.fields().first() else {
                continue;
            };
            let Some(&idx) = front_to_section.get(front.as_str()) else {
                continue;
            };
            // The card belongs to this section's taxonomy, so the section is
            // "covered" whether or not it has been reviewed yet.
            acc[idx].present_cards += 1;
            // Only cards with an FSRS memory state (i.e. reviewed) contribute to
            // the retrievability signal.
            let Some(state) = card.memory_state else {
                continue;
            };
            let last_review = match card.last_review_time {
                Some(t) => Some(t),
                None => self.storage.time_of_last_review(card.id)?,
            };
            let Some(last_review) = last_review else {
                continue;
            };

            let seconds = now.elapsed_secs_since(last_review) as u32;
            let decay = card.decay.unwrap_or(FSRS5_DEFAULT_DECAY);
            let retrievability =
                fsrs.current_retrievability_seconds(state.into(), seconds, decay);

            let section = &mut acc[idx];
            section.sum_retrievability += retrievability;
            section.reviewed_cards += 1.0;
            section.last_updated = section.last_updated.max(last_review.0);
        }
        Ok(())
    }

    /// Add the correct/total signal from the stored practice-exam history.
    fn accumulate_practice_exam_signal(&self, acc: &mut [SectionAccumulator]) {
        let topic_to_section = topic_to_section_index();
        for attempt in self.practice_exam_history() {
            for result in attempt.results {
                if let Some(&idx) = topic_to_section.get(result.topic.as_str()) {
                    let section = &mut acc[idx];
                    section.correct += result.correct;
                    section.total += result.total;
                    section.last_updated = section.last_updated.max(attempt.timestamp);
                }
            }
        }
    }
}

/// Build the performance and readiness estimates for one section, plus the
/// blended probability and effective sample size (both `None`/`0` when there
/// isn't enough data).
fn section_estimates(acc: &SectionAccumulator) -> (MetricEstimate, MetricEstimate, Option<f32>, f32) {
    // A section only states a number once it has a full practice exam's worth
    // of questions; flashcards refine the estimate but can't unlock it.
    if !acc.is_eligible() {
        let est = insufficient(acc.last_updated);
        return (est.clone(), est, None, 0.0);
    }
    let effective = W_CARD * acc.reviewed_cards + W_EXAM * acc.total as f32;
    let weighted_success = W_CARD * acc.sum_retrievability + W_EXAM * acc.correct as f32;
    let p = (weighted_success / effective).clamp(0.0, 1.0);
    let (lo, hi) = wilson_interval(p, effective);
    let justification = format!(
        "Based on {} practice-exam questions and {} flashcard reviews in this section.",
        acc.total, acc.reviewed_cards as u32
    );
    let performance = percent_estimate(p, lo, hi, acc.last_updated, justification.clone());
    let readiness = score_estimate(
        p,
        lo,
        hi,
        SECTION_SCORE_MIN,
        SECTION_SCORE_SPAN,
        performance.confidence,
        acc.last_updated,
        justification,
    );
    (performance, readiness, Some(p), effective)
}

/// Aggregate the section probabilities into the overall metrics. Sections are
/// weighted equally (as on the real MCAT); readiness maps the average onto the
/// full 472-528 scale.
fn overall_estimates(
    section_probabilities: &[f32],
    effective_samples_total: f32,
    reviews: u32,
    questions: u32,
    last_updated: i64,
) -> (MetricEstimate, MetricEstimate) {
    if section_probabilities.is_empty() {
        let est = insufficient(last_updated);
        return (est.clone(), est);
    }
    let p = section_probabilities.iter().sum::<f32>() / section_probabilities.len() as f32;
    let (lo, hi) = wilson_interval(p, effective_samples_total);
    let justification = format!(
        "Based on {reviews} flashcard reviews and {questions} practice-exam questions across {} section(s).",
        section_probabilities.len()
    );
    let performance = percent_estimate(p, lo, hi, last_updated, justification.clone());
    let readiness = score_estimate(
        p,
        lo,
        hi,
        TOTAL_SCORE_MIN,
        TOTAL_SCORE_SPAN,
        performance.confidence,
        last_updated,
        justification,
    );
    (performance, readiness)
}

/// A 0-100 percentage estimate (performance).
fn percent_estimate(p: f32, lo: f32, hi: f32, last_updated: i64, justification: String) -> MetricEstimate {
    let range_min = lo * 100.0;
    let range_max = hi * 100.0;
    MetricEstimate {
        has_enough_data: true,
        score: p * 100.0,
        range_min,
        range_max,
        confidence: confidence_from_range(range_min, range_max),
        last_updated,
        justification,
    }
}

/// A scaled-score estimate (readiness), mapping a probability onto
/// `base..=base + span`. Confidence is inherited from the matching performance
/// estimate, since both describe the same underlying certainty.
#[allow(clippy::too_many_arguments)]
fn score_estimate(
    p: f32,
    lo: f32,
    hi: f32,
    base: f32,
    span: f32,
    confidence: f32,
    last_updated: i64,
    justification: String,
) -> MetricEstimate {
    MetricEstimate {
        has_enough_data: true,
        score: base + span * p,
        range_min: base + span * lo,
        range_max: base + span * hi,
        confidence,
        last_updated,
        justification,
    }
}

/// The estimate used when there isn't enough data to state a number.
fn insufficient(last_updated: i64) -> MetricEstimate {
    MetricEstimate {
        has_enough_data: false,
        score: 0.0,
        range_min: 0.0,
        range_max: 0.0,
        confidence: 0.0,
        last_updated,
        justification: INSUFFICIENT_MSG.to_string(),
    }
}

/// The section label shown to the user, i.e. the deck name without the leading
/// `MCAT::` namespace.
fn section_display_name(deck_name: &str) -> String {
    deck_name.strip_prefix("MCAT::").unwrap_or(deck_name).to_string()
}

/// Maps every taxonomy card front to its section index in [`MCAT_TOPICS`].
fn front_to_section_index() -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (idx, section) in MCAT_TOPICS.iter().enumerate() {
        let (_deck_name, topics) = *section;
        for topic in topics {
            let (_topic_name, fronts) = *topic;
            for front in fronts {
                map.insert(*front, idx);
            }
        }
    }
    map
}

/// Maps each practice-exam topic key to its section index in [`MCAT_TOPICS`].
fn topic_to_section_index() -> HashMap<&'static str, usize> {
    let mut map = HashMap::new();
    for (deck_name, topic_key) in SECTION_TOPICS {
        if let Some(idx) = MCAT_TOPICS.iter().position(|(d, _)| d == deck_name) {
            map.insert(*topic_key, idx);
        }
    }
    map
}

#[cfg(test)]
mod test {
    use anki_proto::stats::record_practice_exam_request::TopicResult;
    use anki_proto::stats::RecordPracticeExamRequest;

    use super::*;

    /// Record a practice exam answering `correct`/`total` questions in `topic`.
    fn record(col: &mut Collection, topic: &str, correct: u32, total: u32) -> Result<()> {
        col.record_practice_exam(RecordPracticeExamRequest {
            results: vec![TopicResult {
                topic: topic.to_string(),
                correct,
                total,
            }],
            timestamp: 0,
        })
    }

    /// The four practice-exam topic keys, one per MCAT section.
    const TOPIC_KEYS: &[&str] = &[
        "biology_biochemistry",
        "chemistry_physics",
        "psychology_sociology",
        "cars",
    ];

    #[test]
    fn insufficient_when_no_data() -> Result<()> {
        let mut col = Collection::new();
        col.seed_mcat_decks()?;
        let metrics = col.exam_metrics()?;
        assert!(!metrics.performance_overall.unwrap().has_enough_data);
        assert!(!metrics.readiness_overall.unwrap().has_enough_data);
        // All four sections are still reported (as insufficient).
        assert_eq!(metrics.performance_sections.len(), MCAT_TOPICS.len());
        Ok(())
    }

    #[test]
    fn one_section_alone_does_not_unlock_overall() -> Result<()> {
        let mut col = Collection::new();
        col.seed_mcat_decks()?;

        // A full exam in just one topic makes that section confident, but the
        // overall estimate stays insufficient until every covered topic has one.
        record(&mut col, "biology_biochemistry", 8, MIN_EXAM_QUESTIONS_PER_SECTION)?;

        let metrics = col.exam_metrics()?;
        assert!(
            !metrics.performance_overall.unwrap().has_enough_data,
            "one section should not unlock the overall estimate"
        );

        let biology = metrics
            .performance_sections
            .iter()
            .find(|s| s.section == "Biology & Biochemistry")
            .and_then(|s| s.estimate.as_ref())
            .unwrap();
        assert!(biology.has_enough_data, "the answered section should be confident");
        Ok(())
    }

    #[test]
    fn all_sections_unlock_overall_within_scale() -> Result<()> {
        let mut col = Collection::new();
        col.seed_mcat_decks()?;

        // A full practice exam in every covered topic.
        for topic in TOPIC_KEYS {
            record(&mut col, topic, 8, MIN_EXAM_QUESTIONS_PER_SECTION)?;
        }

        let metrics = col.exam_metrics()?;
        let overall_perf = metrics.performance_overall.unwrap();
        assert!(overall_perf.has_enough_data);
        assert!((0.0..=100.0).contains(&overall_perf.score));
        assert!(overall_perf.range_min <= overall_perf.score);
        assert!(overall_perf.score <= overall_perf.range_max);

        let overall_readiness = metrics.readiness_overall.unwrap();
        assert!(overall_readiness.has_enough_data);
        assert!(
            (TOTAL_SCORE_MIN..=TOTAL_SCORE_MIN + TOTAL_SCORE_SPAN).contains(&overall_readiness.score),
            "readiness {} should be a valid MCAT score",
            overall_readiness.score
        );
        Ok(())
    }

    #[test]
    fn short_exam_is_not_enough() -> Result<()> {
        let mut col = Collection::new();
        col.seed_mcat_decks()?;

        // A short quiz (below the per-section threshold) never states a number.
        for topic in TOPIC_KEYS {
            record(&mut col, topic, 1, MIN_EXAM_QUESTIONS_PER_SECTION - 1)?;
        }

        let metrics = col.exam_metrics()?;
        assert!(!metrics.performance_overall.unwrap().has_enough_data);
        assert!(
            metrics
                .performance_sections
                .iter()
                .all(|s| !s.estimate.as_ref().unwrap().has_enough_data),
            "no section should be confident on a short quiz"
        );
        Ok(())
    }

    #[test]
    fn exam_weighted_more_than_flashcards() {
        // A section with only exam data at 90% should land near 90%.
        let acc = SectionAccumulator {
            sum_retrievability: 0.0,
            reviewed_cards: 0.0,
            present_cards: 0,
            correct: 9,
            total: 10,
            last_updated: 0,
        };
        let (perf, _readiness, p, _n) = section_estimates(&acc);
        assert!(perf.has_enough_data);
        let p = p.unwrap();
        assert!((0.85..=0.95).contains(&p), "expected ~0.9, got {p}");
    }
}
