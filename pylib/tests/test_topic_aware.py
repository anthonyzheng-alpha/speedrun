# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import time

# Import the test helper first so the `anki` namespace package is entered via
# `anki.collection`, avoiding an import cycle that occurs if `anki.cards` is the
# first submodule imported.
from tests.shared import getEmptyCol

from anki.cards import FSRSMemoryState
from anki.consts import CARD_TYPE_REV, QUEUE_TYPE_REV
from anki.decks import UpdateDeckConfigs, UpdateDeckConfigsMode


def _good_interval(col, card_id) -> int:
    states = col._backend.get_scheduling_states(card_id)
    return states.good.normal.review.scheduled_days


def test_topic_aware_scheduling_shortens_weak_topic():
    col = getEmptyCol()
    # Enable FSRS for the collection.
    col.set_config("fsrs", True)

    # Create a mature FSRS review card.
    note = col.newNote()
    note["Front"] = "q"
    note["Back"] = "a"
    col.addNote(note)
    card = note.cards()[0]
    card.type = CARD_TYPE_REV
    card.queue = QUEUE_TYPE_REV
    card.ivl = 1
    card.reps = 1
    card.memory_state = FSRSMemoryState(stability=100.0, difficulty=5.0)
    col.update_card(card)

    # Seed a weak history for the card's deck: 10 recent "Again" reviews.
    now_ms = int(time.time() * 1000)
    for i in range(10):
        col.db.execute(
            "insert into revlog (id, cid, usn, ease, ivl, lastIvl, factor, time, type)"
            " values (?, ?, 0, 1, 1, 1, 0, 0, 1)",
            now_ms + i,
            card.id,
        )

    # With the feature off (default), record the baseline interval.
    baseline = _good_interval(col, card.id)
    assert baseline > 0

    # Enable topic-aware scheduling on the deck's preset via the modern API.
    data = col.decks.get_deck_configs_for_update(card.did)
    current_id = data.current_deck.config_id
    conf = next(c.config for c in data.all_config if c.config.id == current_id)
    conf.config.topic_aware_scheduling = True
    req = UpdateDeckConfigs(
        target_deck_id=card.did,
        mode=UpdateDeckConfigsMode.UPDATE_DECK_CONFIGS_MODE_NORMAL,
        fsrs=True,
    )
    req.configs.append(conf)
    col.decks.update_deck_configs(req)

    # The weak topic should now be brought back sooner.
    adjusted = _good_interval(col, card.id)
    assert adjusted < baseline
