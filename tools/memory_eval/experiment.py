"""Three-build experiment: does the AI practice-problems feature improve performance?

This harness runs the study protocol from the project ``README.md`` ("Study
feature testing") as three arms of a controlled experiment on synthetic students
(the "models"), and reports the **performance metric = accuracy on a final
held-out MCAT exam** for each build.

Arms (the same cohort is run through all three, so the comparison is paired):

* Build 1 - 20 min studying (own or built-in deck) + an AI practice exam.
* Build 2 - 20 min studying (own or built-in deck), no practice exam.
* Build 3 - 20 min studying (built-in deck only), no practice exam.

Model of the 20-minute study block
----------------------------------
Each synthetic student starts with a latent per-section *mastery* ``m0`` (their
prior knowledge, from ``students.py``). Studying is a sequence of retrieval
events; each event nudges mastery up with diminishing returns::

    m <- m + eta * (1 - m)      =>   after n events:  m = 1 - (1 - m0)(1 - eta)^n

* **Flashcards** use rate ``eta_card``.
* **Practice problems** (Build 1 only, extra events after studying) use
  ``eta_exam = exam_boost * eta_card``. The default ``exam_boost = 3`` is taken
  directly from the app's own weighting of a practice-exam question vs a
  flashcard review (``W_EXAM = 3`` vs ``W_CARD = 1`` in
  ``rslib/src/stats/metrics.rs``): exam-style retrieval is a stronger signal and
  a deeper form of practice.
* **Deck freedom**: builds that can make their own deck (1 and 2) spend study
  events preferentially on their *weak* sections (allocation proportional to
  ``1 - m0``); the built-in-only build (3) splits study evenly. This is what
  lets Build 2 modestly beat Build 3.

The boosted mastery becomes the per-card ceiling ``C``; the existing FSRS
spaced-review simulation (``simulate._simulate_card``) is then reused so
exam-day retrievability/forgetting still applies. Final true recall is
``C * R_exam`` and every final-exam question is a Bernoulli draw at that recall.

The *direction* and *mechanism* of the effects reflect the app's design; the
effect *magnitude* depends on the learning-rate knobs, which are all CLI flags.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field

import numpy as np

from data import SECTION_LABELS, DeckData, load_deck_data
from fsrs_model import FsrsEngine
from simulate import _iter_cards, _review_days, _simulate_card
from students import Student, StudentConfig, generate_students


@dataclass(frozen=True)
class Arm:
    """One experimental build."""

    build: int
    name: str
    # Can the learner make their own deck (target weak sections)?
    own_deck: bool
    # Does the learner take an AI practice exam after studying (Build 1)?
    practice_exam: bool


ARMS: list[Arm] = [
    Arm(1, "Build 1: study + AI practice exam", own_deck=True, practice_exam=True),
    Arm(2, "Build 2: study only (own or built-in deck)", own_deck=True, practice_exam=False),
    Arm(3, "Build 3: study only (built-in deck)", own_deck=False, practice_exam=False),
]


@dataclass
class ExperimentConfig:
    """Knobs for the study/learning model. All exposed as CLI flags."""

    study_minutes: float = 20.0
    # Flashcards reviewed per minute of studying (a review roughly every 20s).
    cards_per_min: float = 3.0
    # Per-flashcard learning rate (fraction of the remaining gap to mastery=1).
    eta_card: float = 0.05
    # Practice-problem learning rate as a multiple of eta_card. Default 3 mirrors
    # W_EXAM/W_CARD in rslib/src/stats/metrics.rs.
    exam_boost: float = 3.0
    # Number of AI practice-exam questions answered per section in Build 1.
    practice_questions_per_section: int = 10
    # Questions per section on the final held-out exam (more = tighter CIs).
    questions_per_section: int = 20
    # Per-card mastery noise around the section mastery.
    card_sigma: float = 0.10
    prob_min: float = 0.02
    prob_max: float = 0.98


def _boost(m0: float, n_events: float, eta: float) -> float:
    """Mastery after ``n_events`` retrievals at rate ``eta`` (diminishing returns)."""
    if n_events <= 0.0 or eta <= 0.0:
        return m0
    return 1.0 - (1.0 - m0) * (1.0 - eta) ** n_events


def _allocate(total_events: float, m0_by_key: dict[str, float], own_deck: bool) -> dict[str, float]:
    """Split a study budget across sections.

    Own-deck builds weight toward weak sections (``1 - m0``); the built-in-only
    build splits evenly.
    """
    keys = list(m0_by_key)
    if own_deck:
        weights = {k: max(1e-6, 1.0 - m0_by_key[k]) for k in keys}
    else:
        weights = {k: 1.0 for k in keys}
    denom = sum(weights.values())
    return {k: total_events * weights[k] / denom for k in keys}


def _final_mastery(
    student: Student, arm: Arm, cfg: ExperimentConfig, keys: list[str]
) -> dict[str, float]:
    """Apply the arm's study protocol to produce post-study section mastery."""
    m0 = {k: student.section_mastery[k] for k in keys}

    total_flash = cfg.study_minutes * cfg.cards_per_min
    flash_events = _allocate(total_flash, m0, arm.own_deck)

    eta_exam = cfg.exam_boost * cfg.eta_card
    m_final: dict[str, float] = {}
    for k in keys:
        m = _boost(m0[k], flash_events[k], cfg.eta_card)
        if arm.practice_exam:
            m = _boost(m, cfg.practice_questions_per_section, eta_exam)
        m_final[k] = float(np.clip(m, cfg.prob_min, cfg.prob_max))
    return m_final


def _section_truth(
    deck: DeckData,
    section_key: str,
    ceiling_mean: float,
    student: Student,
    engine: FsrsEngine,
    cfg: ExperimentConfig,
    rng: np.random.Generator,
) -> float:
    """Mean true exam-day recall over the section's cards, given a mastery ceiling.

    Reuses the base harness's FSRS card simulation so spaced review and
    forgetting between the end of studying and exam day still apply.
    """
    section = deck.section_by_key(section_key)
    n_reviews = student.reviews_per_card[section_key]
    span = student.study_span_days[section_key]
    review_days = _review_days(n_reviews, span)
    exam_day = (review_days[-1] if review_days else 0.0) + student.days_before_exam

    truths: list[float] = []
    for _front in _iter_cards(section):
        ceiling = float(np.clip(rng.normal(ceiling_mean, cfg.card_sigma), cfg.prob_min, cfg.prob_max))
        _card, true_recall = _simulate_card(engine, ceiling, review_days, exam_day, rng)
        truths.append(true_recall)
    if not truths:
        return float(np.clip(ceiling_mean, cfg.prob_min, cfg.prob_max))
    return float(np.mean(truths))


def run_experiment(
    deck: DeckData,
    students: list[Student],
    rng: np.random.Generator,
    cfg: ExperimentConfig | None = None,
) -> list[dict]:
    """Run all three arms on the shared cohort; return one row per exam question."""
    cfg = cfg or ExperimentConfig()
    engine = FsrsEngine()
    keys = [s.key for s in deck.sections]
    rows: list[dict] = []

    for arm in ARMS:
        for student in students:
            m_final = _final_mastery(student, arm, cfg, keys)
            for key in keys:
                truth = _section_truth(deck, key, m_final[key], student, engine, cfg, rng)
                for q_idx in range(cfg.questions_per_section):
                    outcome = 1 if rng.random() < truth else 0
                    rows.append(
                        {
                            "build": arm.build,
                            "arm": arm.name,
                            "student_id": student.student_id,
                            "section": key,
                            "section_label": SECTION_LABELS.get(key, key),
                            "question_idx": q_idx,
                            "outcome": outcome,
                            "section_true": truth,
                            "m0": student.section_mastery[key],
                            "m_final": m_final[key],
                            "ability": student.meta.get("ability", float("nan")),
                        }
                    )
    return rows


# --- scoring -------------------------------------------------------------


def wilson_interval(correct: int, total: int, z: float = 1.96) -> tuple[float, float, float]:
    """Return (point, lo, hi) for a proportion using the Wilson score interval.

    Mirrors ``wilson_interval`` in ``rslib/src/stats/envelope.rs`` (the same 95%
    interval the app uses for its honesty envelope).
    """
    if total == 0:
        return (float("nan"), float("nan"), float("nan"))
    p = correct / total
    n = float(total)
    denom = 1.0 + z * z / n
    center = (p + z * z / (2.0 * n)) / denom
    margin = (z * math.sqrt(p * (1.0 - p) / n + z * z / (4.0 * n * n))) / denom
    return (p, max(0.0, center - margin), min(1.0, center + margin))


def summarize(rows: list[dict]) -> dict:
    """Per-build overall + per-section accuracy with Wilson 95% intervals."""
    summary: dict = {}
    builds = sorted({r["build"] for r in rows})
    section_keys = [k for k in dict.fromkeys(r["section"] for r in rows)]

    for build in builds:
        b_rows = [r for r in rows if r["build"] == build]
        correct = sum(r["outcome"] for r in b_rows)
        total = len(b_rows)
        overall = wilson_interval(correct, total)

        by_section: dict[str, tuple[float, float, float]] = {}
        for key in section_keys:
            s_rows = [r for r in b_rows if r["section"] == key]
            by_section[key] = wilson_interval(
                sum(r["outcome"] for r in s_rows), len(s_rows)
            )
        summary[build] = {
            "arm": b_rows[0]["arm"],
            "n": total,
            "overall": overall,
            "by_section": by_section,
        }
    summary["_section_keys"] = section_keys
    return summary


def plot_performance(summary: dict, deck: DeckData, out_path) -> None:
    """Grouped bar chart of the performance metric by build (overall + sections)."""
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    keys = summary["_section_keys"]
    builds = sorted(b for b in summary if isinstance(b, int))
    categories = ["Overall"] + [SECTION_LABELS.get(k, k) for k in keys]

    x = np.arange(len(categories))
    width = 0.8 / len(builds)

    fig, ax = plt.subplots(figsize=(10, 6))
    for i, build in enumerate(builds):
        s = summary[build]
        points = [s["overall"][0]] + [s["by_section"][k][0] for k in keys]
        los = [s["overall"][1]] + [s["by_section"][k][1] for k in keys]
        his = [s["overall"][2]] + [s["by_section"][k][2] for k in keys]
        points = np.array(points) * 100.0
        lower = points - np.array(los) * 100.0
        upper = np.array(his) * 100.0 - points
        offset = (i - (len(builds) - 1) / 2) * width
        bars = ax.bar(
            x + offset,
            points,
            width,
            yerr=[lower, upper],
            capsize=3,
            label=f"Build {build}",
        )
        for bar, value in zip(bars, points):
            ax.text(
                bar.get_x() + bar.get_width() / 2,
                bar.get_height(),
                f"{value:.0f}",
                ha="center",
                va="bottom",
                fontsize=8,
            )

    ax.set_xticks(x)
    ax.set_xticklabels(categories)
    ax.set_ylabel("Performance metric: final-exam accuracy (%)")
    ax.set_ylim(0, 100)
    ax.set_title("AI practice-problems experiment: performance by build")
    ax.legend(title="Study protocol")
    ax.grid(True, axis="y", alpha=0.3)
    fig.tight_layout()
    fig.savefig(out_path, dpi=120)
    plt.close(fig)


def main(argv: list[str] | None = None) -> int:
    import argparse
    from pathlib import Path

    from score import write_csv

    p = argparse.ArgumentParser(
        description="Measure the effect of AI practice problems across three builds."
    )
    p.add_argument("--students", type=int, default=1000, help="Synthetic students per build.")
    p.add_argument("--seed", type=int, default=0, help="RNG seed for reproducibility.")
    p.add_argument("--study-minutes", type=float, default=20.0, help="Study block length (simulated).")
    p.add_argument("--cards-per-min", type=float, default=3.0, help="Flashcards reviewed per minute.")
    p.add_argument("--eta-card", type=float, default=0.05, help="Per-flashcard learning rate.")
    p.add_argument(
        "--exam-boost",
        type=float,
        default=3.0,
        help="Practice-problem learning rate as a multiple of --eta-card "
        "(default 3, matching W_EXAM/W_CARD).",
    )
    p.add_argument(
        "--practice-questions-per-section",
        type=int,
        default=10,
        help="AI practice-exam questions per section in Build 1.",
    )
    p.add_argument(
        "--questions-per-section",
        type=int,
        default=20,
        help="Questions per section on the final held-out exam.",
    )
    p.add_argument("--card-sigma", type=float, default=0.10, help="Per-card mastery noise (std).")
    p.add_argument("--out", type=str, default="out", help="Output directory.")
    p.add_argument(
        "--report",
        type=str,
        default="",
        help="Optional path to write a plain-text summary (e.g. the repo-root log).",
    )
    args = p.parse_args(argv)

    rng = np.random.default_rng(args.seed)
    deck = load_deck_data()
    section_keys = [s.key for s in deck.sections]
    students = generate_students(
        args.students, section_keys, rng, StudentConfig(card_sigma=args.card_sigma)
    )

    cfg = ExperimentConfig(
        study_minutes=args.study_minutes,
        cards_per_min=args.cards_per_min,
        eta_card=args.eta_card,
        exam_boost=args.exam_boost,
        practice_questions_per_section=args.practice_questions_per_section,
        questions_per_section=args.questions_per_section,
        card_sigma=args.card_sigma,
    )

    rows = run_experiment(deck, students, rng, cfg)
    summary = summarize(rows)

    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    write_csv(rows, out_dir / "experiment_results.csv")

    plot_performance(summary, deck, out_dir / "experiment_performance.png")

    report = render_report(summary, deck, cfg, args)
    print(report)
    print(f"\nWrote: {out_dir / 'experiment_performance.png'}")
    print(f"Wrote: {out_dir / 'experiment_results.csv'}")
    if args.report:
        Path(args.report).write_text(report + "\n", encoding="utf-8")
        print(f"Wrote: {args.report}")
    return 0


def render_report(summary: dict, deck: DeckData, cfg: ExperimentConfig, args) -> str:
    """Build the human-readable per-build table and conclusion."""
    keys = summary["_section_keys"]
    lines: list[str] = []
    lines.append("AI practice-problems effectiveness experiment")
    lines.append("=" * 60)
    lines.append(
        f"Students per build: {args.students}   seed: {args.seed}   "
        f"study: {cfg.study_minutes:.0f} min   "
        f"final exam: {cfg.questions_per_section} q/section"
    )
    lines.append(
        f"Learning rates: eta_card={cfg.eta_card}  exam_boost={cfg.exam_boost}x  "
        f"(practice {cfg.practice_questions_per_section} q/section)"
    )
    lines.append("")
    lines.append("Performance metric = accuracy on a final held-out MCAT exam.")
    lines.append("Each cell shows accuracy% [95% Wilson CI].")
    lines.append("")

    header_cols = ["Overall"] + [SECTION_LABELS.get(k, k) for k in keys]
    lines.append(f"{'Build':<38} " + " ".join(f"{c:>16}" for c in header_cols))

    def fmt(triple: tuple[float, float, float]) -> str:
        p, lo, hi = triple
        return f"{p * 100:5.1f} [{lo * 100:4.1f},{hi * 100:4.1f}]"

    for build in sorted(b for b in summary if isinstance(b, int)):
        s = summary[build]
        cells = [fmt(s["overall"])] + [fmt(s["by_section"][k]) for k in keys]
        lines.append(f"{s['arm']:<38} " + " ".join(f"{c:>16}" for c in cells))

    lines.append("")
    p1 = summary[1]["overall"][0]
    p2 = summary[2]["overall"][0]
    p3 = summary[3]["overall"][0]
    lines.append(f"Practice-exam effect (Build 1 - Build 2): {(p1 - p2) * 100:+.1f} pts")
    lines.append(f"Own-deck effect      (Build 2 - Build 3): {(p2 - p3) * 100:+.1f} pts")
    lines.append(f"Total feature effect (Build 1 - Build 3): {(p1 - p3) * 100:+.1f} pts")
    lines.append("")
    best = max((1, 2, 3), key=lambda b: summary[b]["overall"][0])
    lines.append(
        f"Conclusion: Build {best} scores highest. "
        + (
            "AI practice problems improved the performance metric."
            if p1 >= p2 and p1 >= p3
            else "AI practice problems did NOT come out ahead - review the learning knobs."
        )
    )
    return "\n".join(lines)


if __name__ == "__main__":
    raise SystemExit(main())
