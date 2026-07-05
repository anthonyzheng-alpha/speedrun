// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The user's target exam date.
//!
//! The memory model reports the chance of recalling a card *at the exam*, not
//! right now, so it needs to know when the exam is. The date is stored in the
//! collection config (syncing like any other setting). When it isn't set we
//! fall back to a default horizon so a forward-looking number can still be
//! shown.

use crate::ops::Op;
use crate::prelude::*;

/// Config key holding the exam date as a unix timestamp (seconds).
const EXAM_DATE_KEY: &str = "mcatExamDate";
/// Horizon used when the user hasn't set an exam date yet.
pub(crate) const DEFAULT_EXAM_HORIZON_DAYS: i64 = 30;
const SECS_PER_DAY: i64 = 86_400;

impl Collection {
    /// The user's exam date as a unix timestamp (seconds), if configured.
    pub(crate) fn exam_date_secs(&self) -> Option<i64> {
        self.get_config_optional(EXAM_DATE_KEY)
    }

    /// Persist the exam date (unix seconds).
    pub fn set_exam_date(&mut self, secs: i64) -> Result<()> {
        self.transact(Op::UpdateConfig, |col| {
            col.set_config(EXAM_DATE_KEY, &secs)?;
            Ok(())
        })
        .map(|_| ())
    }

    /// The timestamp (unix seconds) at which recall should be evaluated: the
    /// user's exam date when set, otherwise a default horizon from `now`.
    /// The bool is `true` when the date was user-set.
    pub(crate) fn exam_target_secs(&self, now: TimestampSecs) -> (i64, bool) {
        match self.exam_date_secs() {
            Some(secs) => (secs, true),
            None => (now.0 + DEFAULT_EXAM_HORIZON_DAYS * SECS_PER_DAY, false),
        }
    }
}
