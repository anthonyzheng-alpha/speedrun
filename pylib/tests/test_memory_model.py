# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import time

from anki.cards import FSRSMemoryState
from anki.consts import CARD_TYPE_REV, QUEUE_TYPE_REV

# Import the test helper first so the `anki` namespace package is entered via
# `anki.collection`, avoiding an import cycle that occurs if `anki.cards` is the
# first submodule imported.
from tests.shared import getEmptyCol


def _add_basic_note(col):
    note = col.newNote()
    note["Front"] = "q"
    note["Back"] = "a"
    col.addNote(note)
    return note


def test_memory_model_reports_score_with_enough_data():
    col = getEmptyCol()
    col.set_config("fsrs", True)

    note = _add_basic_note(col)
    card = note.cards()[0]
    card.type = CARD_TYPE_REV
    card.queue = QUEUE_TYPE_REV
    card.ivl = 10
    card.reps = 5
    card.memory_state = FSRSMemoryState(stability=100.0, difficulty=5.0)
    col.update_card(card)

    # Seed several rated reviews (ease=3 -> "Good", counts as recalled).
    now_ms = int(time.time() * 1000)
    for i in range(5):
        col.db.execute(
            "insert into revlog (id, cid, usn, ease, ivl, lastIvl, factor, time, type)"
            " values (?, ?, 0, 3, 10, 10, 0, 0, 1)",
            now_ms + i,
            card.id,
        )

    stats = col.card_stats_data(card.id)
    mem = stats.memory_estimate
    assert mem.has_enough_data
    assert 0.0 <= mem.score <= 100.0
    assert mem.range_min <= mem.score <= mem.range_max
    assert 0.0 <= mem.confidence <= 100.0
    assert mem.justification != ""


def test_memory_model_says_when_not_enough_data():
    col = getEmptyCol()
    col.set_config("fsrs", True)

    # A brand-new card with no memory state and no reviews.
    note = _add_basic_note(col)
    card = note.cards()[0]

    stats = col.card_stats_data(card.id)
    mem = stats.memory_estimate
    assert not mem.has_enough_data
    assert mem.justification != ""
