"""Synthetic learners for the memory-model accuracy experiment.

Each student carries two *independent* traits so the memory model can be wrong
(and the Brier score is therefore meaningful):

* ``section_mastery`` - the durable, latent probability that the student truly
  knows the material in a section (the "ceiling" recall when perfectly fresh).
  This is ground truth the model never sees directly.
* diligence (``reviews_per_card`` and ``study_span_days``) - how much and how
  recently they studied. A crammer can have high diligence but low mastery
  (strong short-term recall that FSRS retrievability overrates), while a
  light-but-capable student can have the opposite.

The actual per-card recall, review outcomes and exam outcomes are simulated in
``simulate.py`` using these traits plus the FSRS engine in ``fsrs_model.py``.
"""

from __future__ import annotations

from dataclasses import dataclass, field

import numpy as np


@dataclass
class StudentConfig:
    """Knobs controlling the synthetic cohort. All have sensible defaults."""

    # Overall ability varies student-to-student; each section then varies around
    # that student's mean.
    ability_low: float = 0.35
    ability_high: float = 0.92
    section_sigma: float = 0.12
    # Per-card variation around the section mastery (applied in simulate.py).
    card_sigma: float = 0.10
    # Diligence: how many spaced reviews each card gets, and over how many days.
    reviews_min: int = 3
    reviews_max: int = 12
    study_span_min_days: int = 1
    study_span_max_days: int = 21
    # Probability floor/ceiling so nothing is a dead certainty.
    prob_min: float = 0.02
    prob_max: float = 0.98


@dataclass
class Student:
    student_id: int
    # topic key -> durable mastery (ceiling recall) in [0, 1]
    section_mastery: dict[str, float]
    # topic key -> number of spaced reviews each studied card receives
    reviews_per_card: dict[str, int]
    # topic key -> number of days the reviews are spread across
    study_span_days: dict[str, int]
    # How long before the exam the study block ends, in days (>=0). Larger =
    # more forgetting by exam time.
    days_before_exam: int = 0
    meta: dict = field(default_factory=dict)


def generate_students(
    n: int,
    section_keys: list[str],
    rng: np.random.Generator,
    config: StudentConfig | None = None,
) -> list[Student]:
    cfg = config or StudentConfig()
    students: list[Student] = []

    for i in range(n):
        ability = rng.uniform(cfg.ability_low, cfg.ability_high)
        section_mastery: dict[str, float] = {}
        reviews_per_card: dict[str, int] = {}
        study_span_days: dict[str, int] = {}

        for key in section_keys:
            mastery = float(
                np.clip(
                    rng.normal(ability, cfg.section_sigma),
                    cfg.prob_min,
                    cfg.prob_max,
                )
            )
            section_mastery[key] = mastery
            reviews_per_card[key] = int(rng.integers(cfg.reviews_min, cfg.reviews_max + 1))
            study_span_days[key] = int(
                rng.integers(cfg.study_span_min_days, cfg.study_span_max_days + 1)
            )

        # A gap between the end of studying and the exam, so FSRS decay matters.
        days_before_exam = int(rng.integers(0, 15))

        students.append(
            Student(
                student_id=i,
                section_mastery=section_mastery,
                reviews_per_card=reviews_per_card,
                study_span_days=study_span_days,
                days_before_exam=days_before_exam,
                meta={"ability": ability},
            )
        )

    return students
