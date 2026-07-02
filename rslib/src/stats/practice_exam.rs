// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Persistence for practice-exam results.
//!
//! Practice exams are otherwise stateless UI, but the performance and
//! readiness metrics need a history of past attempts to learn from. We store a
//! bounded list of attempts in the collection config (which syncs like any
//! other setting) rather than adding a dedicated table.

use anki_proto::stats::RecordPracticeExamRequest;
use serde::Deserialize;
use serde::Serialize;

use crate::ops::Op;
use crate::prelude::*;

/// Config key holding the JSON-encoded practice-exam history.
const PRACTICE_EXAM_HISTORY_KEY: &str = "mcatPracticeExamHistory";
/// Only the most recent attempts are kept, so the config value stays small.
const MAX_ATTEMPTS: usize = 50;

/// One topic's tally within a single practice-exam attempt.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct PracticeExamTopicResult {
    pub topic: String,
    pub correct: u32,
    pub total: u32,
}

/// A completed practice exam, broken down by topic.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct PracticeExamAttempt {
    /// Unix timestamp (seconds) the exam was completed.
    pub timestamp: i64,
    pub results: Vec<PracticeExamTopicResult>,
}

impl Collection {
    /// The stored practice-exam attempts, oldest first. Empty when none have
    /// been recorded (or the stored value can't be parsed).
    pub(crate) fn practice_exam_history(&self) -> Vec<PracticeExamAttempt> {
        self.get_config_optional(PRACTICE_EXAM_HISTORY_KEY)
            .unwrap_or_default()
    }

    /// Append a completed practice exam to the stored history, capping the
    /// history to the most recent [`MAX_ATTEMPTS`].
    pub fn record_practice_exam(&mut self, input: RecordPracticeExamRequest) -> Result<()> {
        let timestamp = if input.timestamp > 0 {
            input.timestamp
        } else {
            TimestampSecs::now().0
        };
        let results = input
            .results
            .into_iter()
            .map(|r| PracticeExamTopicResult {
                topic: r.topic,
                correct: r.correct,
                total: r.total,
            })
            .collect();

        let mut history = self.practice_exam_history();
        history.push(PracticeExamAttempt { timestamp, results });
        if history.len() > MAX_ATTEMPTS {
            let excess = history.len() - MAX_ATTEMPTS;
            history.drain(0..excess);
        }

        self.transact(Op::UpdateConfig, |col| {
            col.set_config(PRACTICE_EXAM_HISTORY_KEY, &history)?;
            Ok(())
        })
        .map(|_| ())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn attempt(topic: &str, correct: u32, total: u32) -> RecordPracticeExamRequest {
        use anki_proto::stats::record_practice_exam_request::TopicResult;
        RecordPracticeExamRequest {
            results: vec![TopicResult {
                topic: topic.to_string(),
                correct,
                total,
            }],
            timestamp: 0,
        }
    }

    #[test]
    fn history_round_trips_and_caps() -> Result<()> {
        let mut col = Collection::new();
        assert!(col.practice_exam_history().is_empty());

        col.record_practice_exam(attempt("biology_biochemistry", 3, 4))?;
        let history = col.practice_exam_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].results[0].correct, 3);
        assert!(history[0].timestamp > 0, "timestamp defaults to now");

        for _ in 0..MAX_ATTEMPTS + 5 {
            col.record_practice_exam(attempt("cars", 1, 2))?;
        }
        assert_eq!(col.practice_exam_history().len(), MAX_ATTEMPTS);
        Ok(())
    }
}
