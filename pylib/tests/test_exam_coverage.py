# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from anki.consts import CARD_TYPE_REV, QUEUE_TYPE_REV

# Import the test helper first so the `anki` namespace package is entered via
# `anki.collection` (see test_memory_model for the same pattern).
from tests.shared import getEmptyCol


def test_exam_coverage_zero_on_empty_collection():
    col = getEmptyCol()
    cov = col.exam_coverage()
    assert cov.topics_total == 0
    assert cov.topics_covered == 0
    assert cov.overall_percent == 0.0
    # A section is still reported for each scored part of the exam.
    assert len(cov.sections) == 4


def test_exam_coverage_increases_after_mastering_a_topic():
    col = getEmptyCol()

    # Cards are matched to a topic purely by their front text, so the deck they
    # live in does not matter; the default deck is fine here.
    note = col.newNote()
    note["Front"] = "What are the four levels of protein structure?"
    note["Back"] = "a"
    col.addNote(note)

    # The topic now has a card present, but it hasn't been studied.
    before = col.exam_coverage()
    assert before.topics_total >= 1
    assert before.topics_covered == 0
    assert before.overall_percent == 0.0

    # Mature the card (review state, 21+ day interval).
    card = note.cards()[0]
    card.type = CARD_TYPE_REV
    card.queue = QUEUE_TYPE_REV
    card.ivl = 21
    col.update_card(card)

    after = col.exam_coverage()
    assert after.topics_total == before.topics_total
    assert after.topics_covered >= 1
    assert after.overall_percent > 0.0

    biology = next(s for s in after.sections if s.section == "Biology & Biochemistry")
    assert biology.topics_covered >= 1
    assert biology.percent > 0.0
