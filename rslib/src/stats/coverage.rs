// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! MCAT exam-coverage reporting.
//!
//! Reports how much of the exam the user has studied so far, expressed as the
//! share of *topics* that are fully covered. Each seeded MCAT card is assigned
//! to a topic by the [`MCAT_TOPICS`](crate::collection::mcat::MCAT_TOPICS)
//! taxonomy; a topic is "covered" once every one of its cards present in the
//! collection has been mastered. The result is broken down by the four scored
//! sections of the exam plus an overall figure, which is what the deck browser
//! home screen displays.

use std::collections::HashMap;
use std::collections::HashSet;

use anki_proto::stats::exam_coverage_response::SectionCoverage;
use anki_proto::stats::ExamCoverageResponse;

use crate::card::CardType;
use crate::collection::mcat::MCAT_TOPICS;
use crate::prelude::*;
use crate::search::SortMode;

/// Minimum interval (in days) at which a review card counts as "mastered" for
/// coverage purposes. This matches Anki's standard notion of a "mature" card.
const MASTERED_INTERVAL_DAYS: u32 = 21;

/// Whether a card counts as mastered: a review-state card whose interval has
/// reached the mature threshold. Learning/relearning/new cards never count.
fn is_mastered(ctype: CardType, interval_days: u32) -> bool {
    matches!(ctype, CardType::Review) && interval_days >= MASTERED_INTERVAL_DAYS
}

/// A topic counts toward its section only when at least one of its cards is
/// present, and it is covered only once every present card is mastered.
fn topic_covered(present: u32, mastered: u32) -> bool {
    present > 0 && mastered == present
}

/// `covered / total * 100`, guarding against division by zero.
fn section_percent(topics_covered: u32, topics_total: u32) -> f32 {
    if topics_total == 0 {
        0.0
    } else {
        topics_covered as f32 / topics_total as f32 * 100.0
    }
}

/// The section label shown to the user, i.e. the deck name without the leading
/// `MCAT::` namespace.
fn section_display_name(deck_name: &str) -> String {
    deck_name
        .strip_prefix("MCAT::")
        .unwrap_or(deck_name)
        .to_string()
}

impl Collection {
    /// Compute how much of the MCAT exam has been studied, overall and per
    /// section. See the module docs for the definition of coverage.
    pub fn exam_coverage(&mut self) -> Result<ExamCoverageResponse> {
        // Cards are matched to the taxonomy purely by their front text, so
        // coverage is independent of how the user organizes their decks.
        let mastered_by_front = self.mastered_fronts_for_taxonomy()?;

        let mut sections = Vec::with_capacity(MCAT_TOPICS.len());
        let mut overall_total = 0u32;
        let mut overall_covered = 0u32;

        for section in MCAT_TOPICS {
            let (deck_name, topics) = *section;

            let mut topics_total = 0u32;
            let mut topics_covered = 0u32;
            for topic in topics {
                let (_topic_name, fronts) = *topic;
                let mut present = 0u32;
                let mut mastered = 0u32;
                for front in fronts {
                    if let Some(card_mastered) = mastered_by_front.get(*front) {
                        present += 1;
                        if *card_mastered {
                            mastered += 1;
                        }
                    }
                }
                if present > 0 {
                    topics_total += 1;
                    if topic_covered(present, mastered) {
                        topics_covered += 1;
                    }
                }
            }

            overall_total += topics_total;
            overall_covered += topics_covered;

            sections.push(SectionCoverage {
                section: section_display_name(deck_name),
                topics_total,
                topics_covered,
                percent: section_percent(topics_covered, topics_total),
            });
        }

        Ok(ExamCoverageResponse {
            sections,
            overall_percent: section_percent(overall_covered, overall_total),
            topics_total: overall_total,
            topics_covered: overall_covered,
        })
    }

    /// Scan the collection and map each card front that belongs to the MCAT
    /// taxonomy to whether that card is mastered. When several cards share a
    /// front, the entry is mastered only if all of them are, so a topic is not
    /// marked covered while a duplicate lags behind. Cards whose front is not
    /// in the taxonomy are ignored.
    fn mastered_fronts_for_taxonomy(&mut self) -> Result<HashMap<String, bool>> {
        let relevant = taxonomy_fronts();
        let mut mastered_by_front = HashMap::new();
        // An empty search matches every card in the collection.
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
            if !relevant.contains(front.as_str()) {
                continue;
            }
            let card_mastered = is_mastered(card.ctype, card.interval);
            mastered_by_front
                .entry(front.clone())
                .and_modify(|m: &mut bool| *m = *m && card_mastered)
                .or_insert(card_mastered);
        }
        Ok(mastered_by_front)
    }
}

/// The set of every card front referenced by the taxonomy.
fn taxonomy_fronts() -> HashSet<&'static str> {
    let mut fronts = HashSet::new();
    for section in MCAT_TOPICS {
        let (_deck_name, topics) = *section;
        for topic in topics {
            let (_topic_name, topic_fronts) = *topic;
            for front in topic_fronts {
                fronts.insert(*front);
            }
        }
    }
    fronts
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::card::CardQueue;

    #[test]
    fn topic_covered_requires_all_present_cards() {
        assert!(!topic_covered(0, 0), "an empty topic is never covered");
        assert!(
            !topic_covered(3, 2),
            "a partially mastered topic is not covered"
        );
        assert!(topic_covered(3, 3), "all mastered -> covered");
        assert!(topic_covered(1, 1));
    }

    #[test]
    fn section_percent_math() {
        assert_eq!(section_percent(0, 0), 0.0);
        assert_eq!(section_percent(1, 2), 50.0);
        assert_eq!(section_percent(4, 4), 100.0);
    }

    #[test]
    fn mastery_threshold() {
        assert!(!is_mastered(CardType::New, 100));
        assert!(!is_mastered(CardType::Learn, 100));
        assert!(!is_mastered(CardType::Relearn, 100));
        assert!(!is_mastered(CardType::Review, MASTERED_INTERVAL_DAYS - 1));
        assert!(is_mastered(CardType::Review, MASTERED_INTERVAL_DAYS));
        assert!(is_mastered(CardType::Review, MASTERED_INTERVAL_DAYS + 100));
    }

    #[test]
    fn coverage_starts_at_zero_and_grows_as_a_topic_is_mastered() -> Result<()> {
        let mut col = Collection::new();
        col.seed_mcat_decks()?;

        // Nothing has been studied yet, so every section reports 0%.
        let before = col.exam_coverage()?;
        assert_eq!(before.sections.len(), MCAT_TOPICS.len());
        assert!(before.topics_total > 0, "seeded decks should have topics");
        assert_eq!(before.topics_covered, 0);
        assert_eq!(before.overall_percent, 0.0);

        // Mature every card in the first topic of the first section, matching
        // by front text.
        let (_deck_name, topics) = MCAT_TOPICS[0];
        let (_topic_name, fronts) = topics[0];
        let front_set: HashSet<&str> = fronts.iter().copied().collect();
        let cids = col.search_cards("", SortMode::NoOrder)?;
        for cid in cids {
            let mut card = col.storage.get_card(cid)?.unwrap();
            let note = col.storage.get_note(card.note_id)?.unwrap();
            if front_set.contains(note.fields()[0].as_str()) {
                card.ctype = CardType::Review;
                card.queue = CardQueue::Review;
                card.interval = MASTERED_INTERVAL_DAYS;
                col.storage.update_card(&card)?;
            }
        }

        let after = col.exam_coverage()?;
        assert_eq!(after.topics_total, before.topics_total);
        assert!(
            after.topics_covered >= 1,
            "the matured topic should be covered"
        );
        assert!(after.overall_percent > 0.0);
        let first = &after.sections[0];
        assert!(first.topics_covered >= 1);
        assert!(first.percent > 0.0);

        Ok(())
    }
}
