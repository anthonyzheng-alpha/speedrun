"""Load the real MCAT deck + practice-exam structure.

The accuracy simulation stays "built around studied topics" by pulling its
inputs from the same sources the app uses:

* Flashcards per section/subtopic from ``MCAT_TOPICS`` in
  ``rslib/src/collection/mcat.rs`` (parsed, mirroring
  ``tools/extract_mcat_flashcards.py``).
* Practice questions per section from
  ``ts/routes/practice-exam/questions.json`` (optionally merged with a
  generated bank).

Only the *structure* (which cards belong to which section/subtopic, and how
many questions exist per section) matters for the synthetic experiment; the
card/question text is not needed.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path

# tools/memory_eval/ -> speedrun/
SPEEDRUN_ROOT = Path(__file__).resolve().parents[2]
MCAT_RS = SPEEDRUN_ROOT / "rslib" / "src" / "collection" / "mcat.rs"
QUESTIONS_JSON = SPEEDRUN_ROOT / "ts" / "routes" / "practice-exam" / "questions.json"
GENERATED_JSON = SPEEDRUN_ROOT / "ts" / "routes" / "practice-exam" / "generated-questions.json"

# Section deck name -> practice-exam topic key. Mirrors SECTION_TOPICS in
# rslib/src/stats/metrics.rs.
SECTION_TOPIC_KEYS: dict[str, str] = {
    "MCAT::Biology & Biochemistry": "biology_biochemistry",
    "MCAT::Chemistry & Physics": "chemistry_physics",
    "MCAT::Psychology & Sociology": "psychology_sociology",
    "MCAT::Critical Analysis & Reasoning (CARS)": "cars",
}

# Human-friendly short labels for graphs, keyed by topic key.
SECTION_LABELS: dict[str, str] = {
    "biology_biochemistry": "Bio/Biochem",
    "chemistry_physics": "Chem/Phys",
    "psychology_sociology": "Psych/Soc",
    "cars": "CARS",
}


@dataclass
class Topic:
    """A finer-grained subtopic within a section, with its card fronts."""

    name: str
    section_key: str
    card_fronts: list[str] = field(default_factory=list)


@dataclass
class Section:
    """One of the four scored MCAT sections."""

    deck_name: str
    key: str
    topics: list[Topic] = field(default_factory=list)

    @property
    def card_count(self) -> int:
        return sum(len(t.card_fronts) for t in self.topics)


@dataclass
class DeckData:
    sections: list[Section]
    # topic key -> number of practice questions available for that section
    questions_per_section: dict[str, int]

    def section_by_key(self, key: str) -> Section:
        for section in self.sections:
            if section.key == key:
                return section
        raise KeyError(key)


def _slice_mcat_topics_block(text: str) -> str:
    """Return the source text of the ``MCAT_TOPICS`` constant array."""
    start = text.index("const MCAT_TOPICS")
    # Skip past the type annotation (`: &[SectionTopics]`) to the assignment,
    # then take the first '[' of the value literal (`= &[ ... ]`).
    eq_idx = text.index("=", start)
    open_idx = text.index("[", eq_idx)
    depth = 0
    for i in range(open_idx, len(text)):
        ch = text[i]
        if ch == "[":
            depth += 1
        elif ch == "]":
            depth -= 1
            if depth == 0:
                return text[open_idx : i + 1]
    raise ValueError("Unbalanced brackets while parsing MCAT_TOPICS")


def _unescape_rust_str(literal: str) -> str:
    return literal.encode("utf-8").decode("unicode_escape")


def parse_sections(mcat_rs: Path = MCAT_RS) -> list[Section]:
    """Parse ``MCAT_TOPICS`` into sections -> topics -> card fronts.

    The taxonomy nesting is::

        ( "MCAT::Section", &[ ( "Topic name", &[ "front", ... ] ), ... ] )

    We walk the block line by line: a bare ``"MCAT::..."`` line opens a
    section, a subsequent quoted line that is immediately followed by a ``&[``
    opens a topic, and remaining quoted string lines are card fronts.
    """
    block = _slice_mcat_topics_block(mcat_rs.read_text(encoding="utf-8"))
    lines = block.splitlines()

    section_re = re.compile(r'^\s*"(MCAT::[^"]+)"\s*,')
    # A quoted string possibly followed by a comma; may be a topic name or a
    # card front. We disambiguate using the next non-empty line.
    quoted_re = re.compile(r'^\s*"((?:[^"\\]|\\.)*)"\s*,?\s*$')
    topic_open_re = re.compile(r"&\s*\[")

    sections: list[Section] = []
    current_section: Section | None = None
    current_topic: Topic | None = None

    for idx, line in enumerate(lines):
        sec_m = section_re.match(line)
        if sec_m:
            deck_name = sec_m.group(1)
            key = SECTION_TOPIC_KEYS.get(deck_name)
            if key is None:
                # Unknown section; skip until the next recognized one.
                current_section = None
                current_topic = None
                continue
            current_section = Section(deck_name=deck_name, key=key)
            sections.append(current_section)
            current_topic = None
            continue

        if current_section is None:
            continue

        q_m = quoted_re.match(line)
        if not q_m:
            continue
        value = _unescape_rust_str(q_m.group(1))

        # Look at the remainder of this line and the next line to decide
        # whether this quoted string introduces a topic (name followed by an
        # opening `&[`).
        rest_of_line = line[q_m.end(1) :]
        next_line = lines[idx + 1] if idx + 1 < len(lines) else ""
        introduces_topic = bool(topic_open_re.search(rest_of_line)) or bool(
            topic_open_re.search(next_line)
        )

        if introduces_topic and current_topic is None:
            current_topic = Topic(name=value, section_key=current_section.key)
            current_section.topics.append(current_topic)
        elif introduces_topic and current_topic is not None and not current_topic.card_fronts:
            # Defensive: two topic-openers in a row (shouldn't happen).
            current_topic = Topic(name=value, section_key=current_section.key)
            current_section.topics.append(current_topic)
        else:
            if current_topic is None:
                continue
            # A `]` on the following line closes the current topic's card list.
            current_topic.card_fronts.append(value)
            if "]" in next_line and "&[" not in next_line:
                current_topic = None

    return sections


def _count_questions(path: Path) -> dict[str, int]:
    counts: dict[str, int] = {}
    if not path.exists():
        return counts
    data = json.loads(path.read_text(encoding="utf-8"))
    for question in data.get("questions", []):
        topic = question.get("topic")
        if topic:
            counts[topic] = counts.get(topic, 0) + 1
    return counts


def load_deck_data(include_generated: bool = False) -> DeckData:
    sections = parse_sections()
    if not sections:
        raise RuntimeError(
            f"Parsed no MCAT sections from {MCAT_RS}; the taxonomy format may have changed."
        )

    questions = _count_questions(QUESTIONS_JSON)
    if include_generated:
        for topic, n in _count_questions(GENERATED_JSON).items():
            questions[topic] = questions.get(topic, 0) + n

    # Ensure every section has an entry (0 if no questions authored yet).
    for section in sections:
        questions.setdefault(section.key, 0)

    return DeckData(sections=sections, questions_per_section=questions)


if __name__ == "__main__":
    deck = load_deck_data()
    for section in deck.sections:
        print(
            f"{section.key:22s} cards={section.card_count:3d} "
            f"topics={len(section.topics):2d} "
            f"questions={deck.questions_per_section.get(section.key, 0)}"
        )
