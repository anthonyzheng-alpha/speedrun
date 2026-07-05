"""Brier scoring and calibration graphs for the memory-model simulation."""

from __future__ import annotations

import csv
from pathlib import Path

import numpy as np

# Use a non-interactive backend so the harness runs headless.
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402


def brier_score(forecasts: np.ndarray, outcomes: np.ndarray) -> float:
    """Mean squared error between forecast probabilities and 0/1 outcomes."""
    if forecasts.size == 0:
        return float("nan")
    return float(np.mean((forecasts - outcomes) ** 2))


def _arrays(rows: list[dict]) -> tuple[np.ndarray, np.ndarray]:
    forecasts = np.array([r["forecast"] for r in rows], dtype=float)
    outcomes = np.array([r["outcome"] for r in rows], dtype=float)
    return forecasts, outcomes


def summarize(rows: list[dict]) -> dict:
    forecasts, outcomes = _arrays(rows)
    overall = brier_score(forecasts, outcomes)

    by_section: dict[str, float] = {}
    counts: dict[str, int] = {}
    sections = sorted({r["section"] for r in rows})
    for key in sections:
        sec_rows = [r for r in rows if r["section"] == key]
        f, o = _arrays(sec_rows)
        by_section[key] = brier_score(f, o)
        counts[key] = len(sec_rows)

    return {
        "overall_brier": overall,
        "by_section": by_section,
        "counts": counts,
        "n": len(rows),
        "base_rate": float(np.mean(outcomes)) if outcomes.size else float("nan"),
    }


def _reliability_bins(
    forecasts: np.ndarray, outcomes: np.ndarray, n_bins: int = 10
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Return (mean_forecast, observed_freq, count) per non-empty decile bin."""
    edges = np.linspace(0.0, 1.0, n_bins + 1)
    idx = np.clip(np.digitize(forecasts, edges[1:-1]), 0, n_bins - 1)
    mean_f, obs, count = [], [], []
    for b in range(n_bins):
        mask = idx == b
        c = int(mask.sum())
        if c == 0:
            continue
        mean_f.append(float(forecasts[mask].mean()))
        obs.append(float(outcomes[mask].mean()))
        count.append(c)
    return np.array(mean_f), np.array(obs), np.array(count)


def plot_reliability(rows: list[dict], out_path: Path, title: str, n_bins: int = 10) -> None:
    forecasts, outcomes = _arrays(rows)
    brier = brier_score(forecasts, outcomes)
    mean_f, obs, count = _reliability_bins(forecasts, outcomes, n_bins)

    fig, (ax, ax_hist) = plt.subplots(
        2, 1, figsize=(7, 8), gridspec_kw={"height_ratios": [3, 1]}, sharex=True
    )

    ax.plot([0, 1], [0, 1], linestyle="--", label="Perfect calibration")
    ax.plot(mean_f, obs, marker="o", label="Memory model")
    ax.set_ylabel("Observed recall (fraction correct)")
    ax.set_ylim(0, 1)
    ax.set_xlim(0, 1)
    ax.set_title(f"{title}\nBrier score = {brier:.4f}  (n = {len(rows):,})")
    ax.legend(loc="upper left")
    ax.grid(True, alpha=0.3)

    ax_hist.bar(mean_f, count, width=1.0 / n_bins * 0.9, align="center")
    ax_hist.set_xlabel("Predicted memory recall")
    ax_hist.set_ylabel("Forecasts")
    ax_hist.grid(True, alpha=0.3)

    fig.tight_layout()
    fig.savefig(out_path, dpi=120)
    plt.close(fig)


def plot_brier_by_section(rows: list[dict], out_path: Path, title: str) -> None:
    summary = summarize(rows)
    sections = list(summary["by_section"].keys())
    labels = []
    values = []
    for key in sections:
        sec_rows = [r for r in rows if r["section"] == key]
        labels.append(sec_rows[0]["section_label"] if sec_rows else key)
        values.append(summary["by_section"][key])

    fig, ax = plt.subplots(figsize=(7, 5))
    bars = ax.bar(labels, values)
    ax.axhline(
        summary["overall_brier"],
        linestyle="--",
        label=f"Overall = {summary['overall_brier']:.4f}",
    )
    ax.set_ylabel("Brier score (lower is better)")
    ax.set_title(title)
    ax.legend()
    ax.grid(True, axis="y", alpha=0.3)
    for bar, value in zip(bars, values):
        ax.text(
            bar.get_x() + bar.get_width() / 2,
            bar.get_height(),
            f"{value:.3f}",
            ha="center",
            va="bottom",
        )
    fig.tight_layout()
    fig.savefig(out_path, dpi=120)
    plt.close(fig)


def write_csv(rows: list[dict], out_path: Path) -> None:
    if not rows:
        out_path.write_text("", encoding="utf-8")
        return
    fields = list(rows[0].keys())
    with out_path.open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)
