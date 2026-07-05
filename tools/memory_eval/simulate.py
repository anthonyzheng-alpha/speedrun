"""Run the synthetic study-then-exam simulation and emit scored rows.

Ground-truth generative story (per student x card):

* The card has a durable ceiling recall ``C`` = section mastery + card noise.
* The student studies the card on spaced (backdated) days. At each review the
  chance of recalling is ``C * R`` where ``R`` is the current FSRS
  retrievability (1.0 on the first exposure). Success -> "Good", miss ->
  "Again"; the FSRS state updates accordingly.
* At exam time (a configurable horizon after study), the true recall of the
  card is ``C * R_exam``.

The memory model only sees the review history, from which it predicts recall as
``blend(exam-date retrievability, observed success rate)`` (see
``fsrs_model.predict_memory_recall``). We aggregate per-card predictions to a
per-section forecast (the "predicted memory rate" for that topic) and compare it
to actual outcomes on that section's practice questions, where each question is
answered correctly with probability equal to the section's true recall.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np

from data import SECTION_LABELS, DeckData
from fsrs_model import FsrsEngine, CardMemory, predict_memory_recall, AGAIN, GOOD
from students import Student


@dataclass
class SimConfig:
    exams: int = 5
    questions_per_section: int = 6
    card_sigma: float = 0.10
    prob_min: float = 0.02
    prob_max: float = 0.98
    # "model" (real prediction) or "perfect" (forecast == truth, a calibration
    # control) - used by the sanity checks.
    forecast_mode: str = "model"


def _review_days(n_reviews: int, span_days: int) -> list[float]:
    if n_reviews <= 1:
        return [0.0]
    span = max(span_days, 0)
    return [span * i / (n_reviews - 1) for i in range(n_reviews)]


def _simulate_card(
    engine: FsrsEngine,
    ceiling: float,
    review_days: list[float],
    exam_day: float,
    rng: np.random.Generator,
) -> tuple[CardMemory, float]:
    """Simulate one card's study history; return its state and true exam recall."""
    card = CardMemory()
    for i, day in enumerate(review_days):
        if i == 0:
            recall_prob = ceiling  # fresh exposure, R = 1
        else:
            recall_prob = ceiling * card.retrievability_at(day, engine.decay)
        rating = GOOD if rng.random() < recall_prob else AGAIN
        card.review(engine, rating, day)

    true_exam_recall = ceiling * card.retrievability_at(exam_day, engine.decay)
    return card, true_exam_recall


def run_simulation(
    deck: DeckData,
    students: list[Student],
    rng: np.random.Generator,
    config: SimConfig | None = None,
) -> list[dict]:
    cfg = config or SimConfig()
    engine = FsrsEngine()
    rows: list[dict] = []

    for student in students:
        for section in deck.sections:
            key = section.key
            mastery = student.section_mastery[key]
            n_reviews = student.reviews_per_card[key]
            span = student.study_span_days[key]
            review_days = _review_days(n_reviews, span)
            exam_day = (review_days[-1] if review_days else 0.0) + student.days_before_exam

            forecasts: list[float] = []
            truths: list[float] = []
            for _front in _iter_cards(section):
                ceiling = float(
                    np.clip(
                        rng.normal(mastery, cfg.card_sigma),
                        cfg.prob_min,
                        cfg.prob_max,
                    )
                )
                card, true_recall = _simulate_card(engine, ceiling, review_days, exam_day, rng)
                pred = predict_memory_recall(card, exam_day, engine.decay)
                if pred is None:
                    continue
                forecasts.append(pred)
                truths.append(true_recall)

            if not forecasts:
                continue

            section_forecast = float(np.mean(forecasts))
            section_truth = float(np.mean(truths))
            if cfg.forecast_mode == "perfect":
                section_forecast = section_truth

            # Take several practice exams; each question is correct with
            # probability equal to the section's true recall.
            for exam_idx in range(cfg.exams):
                for q_idx in range(cfg.questions_per_section):
                    outcome = 1 if rng.random() < section_truth else 0
                    rows.append(
                        {
                            "student_id": student.student_id,
                            "section": key,
                            "section_label": SECTION_LABELS.get(key, key),
                            "exam_idx": exam_idx,
                            "question_idx": q_idx,
                            "forecast": section_forecast,
                            "outcome": outcome,
                            "section_true": section_truth,
                            "n_cards": len(forecasts),
                            "ability": student.meta.get("ability", float("nan")),
                        }
                    )

    return rows


def _iter_cards(section) -> list[str]:
    fronts: list[str] = []
    for topic in section.topics:
        fronts.extend(topic.card_fronts)
    return fronts


def _build_parser():
    import argparse

    p = argparse.ArgumentParser(
        description="Simulate synthetic learners and score the memory model's calibration."
    )
    p.add_argument("--students", type=int, default=200, help="Number of synthetic learners.")
    p.add_argument("--seed", type=int, default=0, help="RNG seed for reproducibility.")
    p.add_argument("--exams", type=int, default=5, help="Practice exams each student takes.")
    p.add_argument(
        "--questions-per-section",
        type=int,
        default=6,
        help="Questions per section in each practice exam.",
    )
    p.add_argument(
        "--exam-horizon-days",
        type=int,
        default=-1,
        help="Fixed days from end-of-study to exam for all students. "
        "-1 (default) uses a random per-student gap.",
    )
    p.add_argument("--card-sigma", type=float, default=0.10, help="Per-card mastery noise (std).")
    p.add_argument(
        "--sections",
        type=str,
        default="",
        help="Comma-separated topic keys to include (default: all).",
    )
    p.add_argument(
        "--include-generated",
        action="store_true",
        help="Merge the generated practice-question bank when counting questions.",
    )
    p.add_argument(
        "--control",
        choices=["none", "perfect"],
        default="none",
        help="'perfect' sets forecast==truth as a calibration control.",
    )
    p.add_argument("--out", type=str, default="out", help="Output directory.")
    return p


def main(argv: list[str] | None = None) -> int:
    from pathlib import Path

    from data import load_deck_data
    from score import (
        plot_brier_by_section,
        plot_reliability,
        summarize,
        write_csv,
    )
    from students import StudentConfig, generate_students

    args = _build_parser().parse_args(argv)
    rng = np.random.default_rng(args.seed)

    deck = load_deck_data(include_generated=args.include_generated)
    if args.sections.strip():
        wanted = {s.strip() for s in args.sections.split(",") if s.strip()}
        deck.sections = [s for s in deck.sections if s.key in wanted]
        if not deck.sections:
            raise SystemExit(f"No sections matched {sorted(wanted)}")

    section_keys = [s.key for s in deck.sections]
    student_cfg = StudentConfig(card_sigma=args.card_sigma)
    students = generate_students(args.students, section_keys, rng, student_cfg)

    # Honor a fixed exam horizon when requested.
    if args.exam_horizon_days >= 0:
        for student in students:
            student.days_before_exam = args.exam_horizon_days

    sim_cfg = SimConfig(
        exams=args.exams,
        questions_per_section=args.questions_per_section,
        card_sigma=args.card_sigma,
        forecast_mode="perfect" if args.control == "perfect" else "model",
    )
    rows = run_simulation(deck, students, rng, sim_cfg)

    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    summary = summarize(rows)
    write_csv(rows, out_dir / "results.csv")
    plot_reliability(rows, out_dir / "reliability.png", "Memory model calibration")
    plot_brier_by_section(rows, out_dir / "brier_by_section.png", "Brier score by MCAT section")

    print(f"Students: {args.students}   Rows (practice questions): {summary['n']:,}")
    print(f"Base rate (overall fraction correct): {summary['base_rate']:.3f}")
    print(f"Overall Brier score: {summary['overall_brier']:.4f}")
    print("Per-section Brier:")
    for key, value in summary["by_section"].items():
        label = SECTION_LABELS.get(key, key)
        print(f"  {label:12s} {value:.4f}   (n={summary['counts'][key]:,})")
    print(f"\nWrote: {out_dir / 'reliability.png'}")
    print(f"Wrote: {out_dir / 'brier_by_section.png'}")
    print(f"Wrote: {out_dir / 'results.csv'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
