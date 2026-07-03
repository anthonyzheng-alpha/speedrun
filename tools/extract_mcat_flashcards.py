#!/usr/bin/env python3
"""Extract MCAT flashcards from rslib/src/collection/mcat.rs for audit."""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MCAT_RS = ROOT / "rslib" / "src" / "collection" / "mcat.rs"
OUT = ROOT / "tools" / "mcat_flashcard_audit.json"

# Each MCAT section can be covered by several Jack Westin books; the first entry
# is treated as the primary/default when a single book must be chosen.
JW_BOOKS = {
    "MCAT::Biology & Biochemistry": ["biology", "biochemistry"],
    "MCAT::Chemistry & Physics": ["general-chemistry", "organic-chemistry", "physics"],
    "MCAT::Psychology & Sociology": ["behavioral-sciences"],
    "MCAT::Critical Analysis & Reasoning (CARS)": ["cars"],
}

CHEM_PHYS_KEYWORDS = {
    "organic-chemistry": [
        "enantiomer", "SN1", "SN2", "nucleophile", "electrophile", "carbonyl",
        "aldehyde", "ketone", "alcohol", "IR", "isomer", "resonance", "stereoisomer",
    ],
    "physics": [
        "Newton", "force", "Ohm", "Coulomb", "electric", "capacitor", "resistor",
        "wave", "Doppler", "lens", "mirror", "Snell", "refraction", "Archimedes",
        "Bernoulli", "pendulum", "spring", "kinematic", "work", "momentum", "impulse",
        "pressure", "fluid", "decibel", "wavelength", "frequency",
    ],
}


def suggest_jw_book(deck: str, front: str, back: str) -> str:
    if deck == "MCAT::Chemistry & Physics":
        text = (front + " " + back).lower()
        for book, keywords in CHEM_PHYS_KEYWORDS.items():
            if any(k.lower() in text for k in keywords):
                return book
        return "general-chemistry"
    books = JW_BOOKS.get(deck)
    return books[0] if books else "unknown"


def main() -> None:
    text = MCAT_RS.read_text(encoding="utf-8")
    lines = text.splitlines()

    start = next(i for i, l in enumerate(lines) if "const MCAT_SECTIONS" in l)
    end = next(i for i, l in enumerate(lines) if "const MCAT_TOPICS" in l)
    line_offset = start
    lines = lines[start:end]

    cards = []
    current_deck = None
    i = 0
    while i < len(lines):
        line = lines[i]
        deck_match = re.match(r'\s+"MCAT::[^"]+",', line)
        if deck_match:
            current_deck = deck_match.group(0).strip().strip('",')
            i += 1
            continue
        if current_deck and re.match(r'\s+"[^"]+",', line):
            front = re.match(r'\s+"([^"]+)",', line).group(1)
            if i + 1 < len(lines):
                back_match = re.match(r'\s+"([^"]+)",', lines[i + 1])
                if back_match:
                    back = back_match.group(1)
                    cards.append(
                        {
                            "id": len(cards) + 1,
                            "deck": current_deck,
                            "line": line_offset + i + 1,
                            "front": front,
                            "back": back,
                            "jw_book": suggest_jw_book(current_deck, front, back),
                            "status": "pending",
                        }
                    )
                    i += 2
                    continue
        i += 1

    dup_clusters = {
        "glycolysis net ATP": [
            "What is the net ATP yield of glycolysis per glucose molecule?",
            "What is the net yield of glycolysis per glucose?",
        ],
        "citric acid cycle location": [
            "Where does the citric acid (Krebs) cycle occur in eukaryotes?",
            "Where does the citric acid cycle occur and what does it produce per acetyl-CoA?",
        ],
        "sympathetic vs parasympathetic": [
            "Which neurotransmitters do the two divisions of the autonomic nervous system use?",
            "What is the difference between the sympathetic and parasympathetic nervous systems?",
        ],
        "pH definition": [
            "Define pH in terms of hydrogen ion concentration.",
            "How is pH defined in terms of hydrogen ion concentration?",
        ],
        "Ohm's law": [
            "State Ohm's law.",
            "State Ohm's law and the formula for electrical power.",
        ],
        "enantiomers": [
            "Define an enantiomer.",
            "What are enantiomers and how do they differ physically?",
        ],
    }
    front_to_cluster = {}
    for name, fronts in dup_clusters.items():
        for front in fronts:
            front_to_cluster[front] = name

    n_flagged = 0
    for card in cards:
        if card["front"] in front_to_cluster:
            card["duplicate_cluster"] = front_to_cluster[card["front"]]
            n_flagged += 1

    summary = {
        "total": len(cards),
        "by_deck": {},
        "duplicates_flagged": n_flagged,
    }
    for card in cards:
        summary["by_deck"][card["deck"]] = summary["by_deck"].get(card["deck"], 0) + 1

    OUT.write_text(
        json.dumps({"summary": summary, "cards": cards}, indent=2),
        encoding="utf-8",
    )
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
