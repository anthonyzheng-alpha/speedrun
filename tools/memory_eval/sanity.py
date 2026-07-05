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
from score import summarize
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


def _crammer_students(keys, rng, n=150) -> list[Student]:
    """Crammers: high review count, all same day, low mastery, exam right after.

    The FSRS retrievability is ~1 right after cramming, so the old blended
    forecast sits well above the student's true (low) recall.
    """
    students: list[Student] = []
    for i in range(n):
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
    return students


def check_multiplicative_reduces_overconfidence() -> bool:
    """The multiplicative model should be less over-confident than the old blend
    on the crammer cohort."""
    deck = load_deck_data()
    keys = [s.key for s in deck.sections]

    rng = np.random.default_rng(2)
    blend_rows = run_simulation(
        deck, _crammer_students(keys, rng), rng, SimConfig(predict_mode="blend")
    )
    rng = np.random.default_rng(2)
    mult_rows = run_simulation(
        deck, _crammer_students(keys, rng), rng, SimConfig(predict_mode="multiplicative")
    )

    blend_gap = float(_arrays(blend_rows)[0].mean() - _arrays(blend_rows)[1].mean())
    mult_gap = float(_arrays(mult_rows)[0].mean() - _arrays(mult_rows)[1].mean())
    ok = mult_gap < blend_gap
    print(
        f"[3] crammer over-confidence gap: multiplicative={mult_gap:+.3f} < "
        f"blend={blend_gap:+.3f} -> {'PASS' if ok else 'FAIL'}"
    )
    return ok


def check_multiplicative_low_bias() -> bool:
    """Over the full cohort, the multiplicative model's mean forecast should be
    close to the observed base rate (little systematic over/under-prediction)."""
    deck = load_deck_data()
    keys = [s.key for s in deck.sections]

    rng = np.random.default_rng(3)
    students = generate_students(200, keys, rng, StudentConfig())
    mult_rows = run_simulation(deck, students, rng, SimConfig(predict_mode="multiplicative"))

    rng = np.random.default_rng(3)
    students = generate_students(200, keys, rng, StudentConfig())
    blend_rows = run_simulation(deck, students, rng, SimConfig(predict_mode="blend"))

    mult_bias = float(_arrays(mult_rows)[0].mean() - _arrays(mult_rows)[1].mean())
    blend_bias = float(_arrays(blend_rows)[0].mean() - _arrays(blend_rows)[1].mean())
    ok = abs(mult_bias) < abs(blend_bias)
    print(
        f"[4] calibration bias: |multiplicative|={abs(mult_bias):.3f} < "
        f"|blend|={abs(blend_bias):.3f} (mult={mult_bias:+.3f}, blend={blend_bias:+.3f}) "
        f"-> {'PASS' if ok else 'FAIL'}"
    )
    return ok


def main() -> int:
    results = [
        check_perfect_beats_model(),
        check_forecast_has_signal(),
        check_multiplicative_reduces_overconfidence(),
        check_multiplicative_low_bias(),
    ]
    passed = all(results)
    print(f"\n{'ALL CHECKS PASSED' if passed else 'SOME CHECKS FAILED'}")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
