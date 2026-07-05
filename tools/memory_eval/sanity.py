"""Sanity checks for the memory-model accuracy simulation.

Run directly: ``python sanity.py``. Exits non-zero if any check fails.

Checks:
1. Perfect-calibration control (forecast == truth) achieves a Brier score no
   worse than the model - i.e. the model can't beat the irreducible noise floor.
2. The model's forecasts are positively correlated with observed recall
   (it carries real signal, not a constant guess).
3. A deliberately mis-scheduled "crammer" cohort (many same-day reviews, low
   durable mastery, exam right after cramming) shows over-confidence: mean
   forecast exceeds mean outcome.
"""

from __future__ import annotations

import numpy as np

from data import load_deck_data
from score import brier_score, summarize
from simulate import SimConfig, run_simulation
from students import Student, StudentConfig, generate_students


def _arrays(rows):
    return (
        np.array([r["forecast"] for r in rows], dtype=float),
        np.array([r["outcome"] for r in rows], dtype=float),
    )


def check_perfect_beats_model() -> bool:
    deck = load_deck_data()
    keys = [s.key for s in deck.sections]

    rng = np.random.default_rng(0)
    students = generate_students(150, keys, rng, StudentConfig())
    model_rows = run_simulation(deck, students, rng, SimConfig(forecast_mode="model"))

    rng = np.random.default_rng(0)
    students = generate_students(150, keys, rng, StudentConfig())
    perfect_rows = run_simulation(deck, students, rng, SimConfig(forecast_mode="perfect"))

    model_brier = summarize(model_rows)["overall_brier"]
    perfect_brier = summarize(perfect_rows)["overall_brier"]
    ok = perfect_brier <= model_brier + 1e-9
    print(
        f"[1] perfect control Brier={perfect_brier:.4f} <= model Brier={model_brier:.4f} "
        f"-> {'PASS' if ok else 'FAIL'}"
    )
    return ok


def check_forecast_has_signal() -> bool:
    deck = load_deck_data()
    keys = [s.key for s in deck.sections]
    rng = np.random.default_rng(1)
    students = generate_students(200, keys, rng, StudentConfig())
    rows = run_simulation(deck, students, rng, SimConfig())

    forecasts, outcomes = _arrays(rows)
    corr = float(np.corrcoef(forecasts, outcomes)[0, 1])
    ok = corr > 0.2
    print(f"[2] corr(forecast, outcome)={corr:.3f} > 0.2 -> {'PASS' if ok else 'FAIL'}")
    return ok


def check_crammer_overconfident() -> bool:
    """Crammers: high review count, all same day, low mastery, exam right after.

    The FSRS retrievability is ~1 right after cramming, so the blended forecast
    sits well above the student's true (low) recall -> systematic over-forecast.
    """
    deck = load_deck_data()
    keys = [s.key for s in deck.sections]
    rng = np.random.default_rng(2)

    students: list[Student] = []
    for i in range(150):
        mastery = float(rng.uniform(0.30, 0.50))
        students.append(
            Student(
                student_id=i,
                section_mastery={k: mastery for k in keys},
                reviews_per_card={k: 12 for k in keys},
                study_span_days={k: 1 for k in keys},
                days_before_exam=0,
                meta={"ability": mastery},
            )
        )

    rows = run_simulation(deck, students, rng, SimConfig())
    forecasts, outcomes = _arrays(rows)
    gap = float(forecasts.mean() - outcomes.mean())
    brier = brier_score(forecasts, outcomes)
    ok = gap > 0.03
    print(
        f"[3] crammer over-confidence: mean forecast-outcome={gap:+.3f} "
        f"(Brier={brier:.4f}) > 0.03 -> {'PASS' if ok else 'FAIL'}"
    )
    return ok


def main() -> int:
    results = [
        check_perfect_beats_model(),
        check_forecast_has_signal(),
        check_crammer_overconfident(),
    ]
    passed = all(results)
    print(f"\n{'ALL CHECKS PASSED' if passed else 'SOME CHECKS FAILED'}")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
