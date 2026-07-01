// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Topic-aware scheduling.
//!
//! When enabled (and FSRS is active), review intervals for cards in weaker
//! topics are shortened so that those topics are brought back for review
//! sooner. A "topic" is the card's home deck, and its weakness is estimated
//! from the recent again-rate in the `revlog`.
//!
//! Crucially, this only scales the day-count interval; the FSRS memory state
//! (stability/difficulty) is computed and stored separately, so the underlying
//! FSRS model remains valid and unaffected.

use crate::prelude::*;
use crate::timestamp::TimestampMillis;

/// How far back (in days) to look when estimating a topic's again-rate.
const TOPIC_LOOKBACK_DAYS: i64 = 30;
/// The strongest shortening applied to a fully-failed topic. An again-rate of
/// 1.0 maps to this multiplier; 0.0 maps to 1.0 (no change).
const MIN_TOPIC_MULTIPLIER: f32 = 0.5;
/// Minimum number of recent reviews required before any adjustment is applied.
/// Below this, there isn't enough signal, so the multiplier stays at 1.0.
const MIN_TOPIC_SAMPLES: u32 = 5;

/// Map a topic's recent again-rate (in `0.0..=1.0`) to an interval multiplier
/// in `[MIN_TOPIC_MULTIPLIER, 1.0]`. Higher again-rates (weaker topics) yield
/// smaller multipliers, which shorten intervals. The mapping is linear and
/// never lengthens an interval.
pub(crate) fn again_rate_to_multiplier(again_rate: f32) -> f32 {
    let again_rate = again_rate.clamp(0.0, 1.0);
    (1.0 - again_rate * (1.0 - MIN_TOPIC_MULTIPLIER)).clamp(MIN_TOPIC_MULTIPLIER, 1.0)
}

impl Collection {
    /// Return the topic-aware interval multiplier for the given home deck,
    /// memoizing the result for the duration of the current review session.
    /// Returns `1.0` (no adjustment) when there is insufficient history.
    pub(crate) fn topic_interval_multiplier(&mut self, deck_id: DeckId) -> Result<f32> {
        if let Some(multiplier) = self.state.topic_weakness_cache.get(&deck_id) {
            return Ok(*multiplier);
        }
        let cutoff_ms = TimestampMillis::now().0 - TOPIC_LOOKBACK_DAYS * 86_400 * 1000;
        let (total, again) = self.storage.topic_again_counts(deck_id, cutoff_ms)?;
        let multiplier = if total < MIN_TOPIC_SAMPLES {
            1.0
        } else {
            again_rate_to_multiplier(again as f32 / total as f32)
        };
        self.state.topic_weakness_cache.insert(deck_id, multiplier);
        Ok(multiplier)
    }

    /// Drop any memoized topic multipliers. Called after answering a card so
    /// that freshly recorded reviews are reflected on the next computation.
    pub(crate) fn clear_topic_weakness_cache(&mut self) {
        self.state.topic_weakness_cache.clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::card::CardQueue;
    use crate::card::CardType;
    use crate::card::FsrsMemoryState;
    use crate::config::BoolKey;
    use crate::revlog::RevlogEntry;
    use crate::revlog::RevlogId;
    use crate::revlog::RevlogReviewKind;
    use crate::scheduler::states::CardState;
    use crate::scheduler::states::NormalState;
    use crate::scheduler::states::ReviewState;

    /// Create an FSRS review card in the default deck with the given stability,
    /// returning its id.
    fn review_card(col: &mut Collection, stability: f32) -> CardId {
        let nt = col.get_notetype_by_name("Basic").unwrap().unwrap();
        let mut note = nt.new_note();
        col.add_note(&mut note, DeckId(1)).unwrap();
        let mut card = col
            .storage
            .all_cards_of_note(note.id)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        card.ctype = CardType::Review;
        card.queue = CardQueue::Review;
        card.interval = 1;
        card.reps = 1;
        card.due = 0;
        card.memory_state = Some(FsrsMemoryState {
            stability,
            difficulty: 5.0,
        });
        col.storage.update_card(&card).unwrap();
        card.id
    }

    /// Seed `total` recent review revlog entries for `cid`, `again` of which
    /// were answered with the Again button.
    fn seed_reviews(col: &mut Collection, cid: CardId, total: u32, again: u32) {
        let now = TimestampMillis::now().0;
        for i in 0..total {
            let entry = RevlogEntry {
                id: RevlogId(now + i as i64),
                cid,
                button_chosen: if i < again { 1 } else { 3 },
                review_kind: RevlogReviewKind::Review,
                ..Default::default()
            };
            col.storage.add_revlog_entry(&entry, true).unwrap();
        }
    }

    /// Return the `Good` review state for a card.
    fn good_review(col: &mut Collection, cid: CardId) -> ReviewState {
        match col.get_scheduling_states(cid).unwrap().good {
            CardState::Normal(NormalState::Review(r)) => r,
            other => panic!("expected a review state for Good, got {other:?}"),
        }
    }

    fn fsrs_collection() -> Collection {
        let mut col = Collection::new();
        col.set_config_bool(BoolKey::Fsrs, true, false).unwrap();
        col
    }

    #[test]
    fn topic_aware_off_ignores_weakness() {
        // With the toggle off, a deck with a terrible again-rate must schedule
        // exactly the same as a deck with no history: the app behaves normally.
        let mut weak = fsrs_collection();
        let weak_card = review_card(&mut weak, 100.0);
        seed_reviews(&mut weak, weak_card, 10, 10);
        let weak_good = good_review(&mut weak, weak_card).scheduled_days;

        let mut clean = fsrs_collection();
        let clean_card = review_card(&mut clean, 100.0);
        let clean_good = good_review(&mut clean, clean_card).scheduled_days;

        assert_eq!(
            weak_good, clean_good,
            "with the feature off, recent failures must not change scheduling"
        );
    }

    #[test]
    fn weak_topic_shortens_interval_when_enabled() {
        let mut col = fsrs_collection();
        let cid = review_card(&mut col, 100.0);
        seed_reviews(&mut col, cid, 10, 10); // again-rate 1.0 -> multiplier 0.5

        // Baseline with the feature off.
        let baseline = good_review(&mut col, cid);

        // Enable topic-aware scheduling for the preset.
        col.update_default_deck_config(|c| c.topic_aware_scheduling = true);
        let adjusted = good_review(&mut col, cid);

        assert!(
            adjusted.scheduled_days < baseline.scheduled_days,
            "weak topic should be brought back sooner ({} < {})",
            adjusted.scheduled_days,
            baseline.scheduled_days
        );
        // FSRS memory state must be identical: only the interval is scaled.
        assert_eq!(
            adjusted.memory_state, baseline.memory_state,
            "FSRS memory state must not be affected by topic-aware scheduling"
        );
    }

    #[test]
    fn strong_topic_unchanged_when_enabled() {
        let mut col = fsrs_collection();
        let cid = review_card(&mut col, 100.0);
        seed_reviews(&mut col, cid, 10, 0); // again-rate 0.0 -> multiplier 1.0

        let baseline = good_review(&mut col, cid).scheduled_days;

        col.update_default_deck_config(|c| c.topic_aware_scheduling = true);
        let adjusted = good_review(&mut col, cid).scheduled_days;

        assert_eq!(
            adjusted, baseline,
            "a strong topic (no recent failures) should not be shortened"
        );
    }

    #[test]
    fn topic_multiplier_mapping() {
        // Strong topic (never failed) -> no change.
        assert_eq!(again_rate_to_multiplier(0.0), 1.0);
        // Fully-failed topic -> maximum shortening.
        assert_eq!(again_rate_to_multiplier(1.0), MIN_TOPIC_MULTIPLIER);
        // Halfway -> midpoint between MIN and 1.0.
        assert_eq!(again_rate_to_multiplier(0.5), 0.75);
        // Monotonically non-increasing and always within bounds.
        let mut last = f32::INFINITY;
        for i in 0..=10 {
            let m = again_rate_to_multiplier(i as f32 / 10.0);
            assert!(m <= last, "multiplier should not increase with again-rate");
            assert!((MIN_TOPIC_MULTIPLIER..=1.0).contains(&m));
            last = m;
        }
        // Out-of-range inputs are clamped.
        assert_eq!(again_rate_to_multiplier(-1.0), 1.0);
        assert_eq!(again_rate_to_multiplier(2.0), MIN_TOPIC_MULTIPLIER);
    }
}
