"""Self-contained FSRS-5 engine + the app's memory-model prediction.

We deliberately avoid a third-party dependency so the harness runs today with
only numpy/matplotlib. The forgetting curve and default decay match the Rust
``fsrs`` crate the app uses (``FSRS5_DEFAULT_DECAY = -0.5``), so retrievability
lines up with ``current_retrievability_seconds`` in
``rslib/src/stats/card.rs``. Stability/difficulty updates use the standard
FSRS-5 equations with the published default parameters.

The prediction itself is copied verbatim from
``rslib/src/stats/memory.rs``: blend the exam-date retrievability with the
observed recall rate, weighting the observed rate more as reviews accumulate,
and only report a number once there are at least ``MIN_MEMORY_SAMPLES`` rated
reviews (and a memory state exists).
"""

from __future__ import annotations

from dataclasses import dataclass

# FSRS-5 default parameters (19 weights), matching the fsrs crate defaults.
DEFAULT_PARAMS: tuple[float, ...] = (
    0.40255, 1.18385, 3.173, 15.69105, 7.1949, 0.5345, 1.4604, 0.0046,
    1.54575, 0.1192, 1.01925, 1.9395, 0.11, 0.29605, 2.2698, 0.2315,
    2.9898, 0.51655, 0.6621,
)

# FSRS-5 default decay and the derived forgetting-curve factor.
FSRS5_DEFAULT_DECAY: float = -0.5
FACTOR: float = 0.9 ** (1.0 / FSRS5_DEFAULT_DECAY) - 1.0  # = 19/81 for decay -0.5

SECONDS_PER_DAY: float = 86_400.0
STABILITY_MIN: float = 0.01
DIFFICULTY_MIN: float = 1.0
DIFFICULTY_MAX: float = 10.0

# Mirror of MIN_MEMORY_SAMPLES in rslib/src/stats/memory.rs.
MIN_MEMORY_SAMPLES: int = 3

# Rating buttons, matching Anki (1=Again .. 4=Easy).
AGAIN, HARD, GOOD, EASY = 1, 2, 3, 4


def retrievability(stability: float, elapsed_days: float, decay: float = FSRS5_DEFAULT_DECAY) -> float:
    """FSRS power forgetting curve: chance of recall after ``elapsed_days``."""
    if stability <= 0.0:
        return 0.0
    elapsed_days = max(elapsed_days, 0.0)
    factor = 0.9 ** (1.0 / decay) - 1.0
    return (1.0 + factor * elapsed_days / stability) ** decay


def retrievability_seconds(
    stability: float, elapsed_seconds: float, decay: float = FSRS5_DEFAULT_DECAY
) -> float:
    """Seconds-based variant, mirroring ``current_retrievability_seconds``."""
    return retrievability(stability, elapsed_seconds / SECONDS_PER_DAY, decay)


def _clamp(value: float, low: float, high: float) -> float:
    return max(low, min(high, value))


class FsrsEngine:
    """FSRS-5 stability/difficulty updates with default parameters."""

    def __init__(self, params: tuple[float, ...] = DEFAULT_PARAMS, decay: float = FSRS5_DEFAULT_DECAY):
        self.w = params
        self.decay = decay

    # -- initial state (first rating) ------------------------------------
    def init_stability(self, rating: int) -> float:
        return max(self.w[rating - 1], STABILITY_MIN)

    def init_difficulty(self, rating: int) -> float:
        d = self.w[4] - _exp(self.w[5] * (rating - 1)) + 1.0
        return _clamp(d, DIFFICULTY_MIN, DIFFICULTY_MAX)

    # -- subsequent updates ----------------------------------------------
    def next_difficulty(self, difficulty: float, rating: int) -> float:
        delta = -self.w[6] * (rating - 3)
        damped = difficulty + delta * (10.0 - difficulty) / 9.0
        # Mean reversion toward the "easy" initial difficulty.
        target = self.w[4] - _exp(self.w[5] * (EASY - 1)) + 1.0
        reverted = self.w[7] * target + (1.0 - self.w[7]) * damped
        return _clamp(reverted, DIFFICULTY_MIN, DIFFICULTY_MAX)

    def next_stability(self, difficulty: float, stability: float, r: float, rating: int) -> float:
        if rating == AGAIN:
            return self._stability_after_failure(difficulty, stability, r)
        return self._stability_after_success(difficulty, stability, r, rating)

    def _stability_after_success(
        self, difficulty: float, stability: float, r: float, rating: int
    ) -> float:
        hard_penalty = self.w[15] if rating == HARD else 1.0
        easy_bonus = self.w[16] if rating == EASY else 1.0
        alpha = (
            _exp(self.w[8])
            * (11.0 - difficulty)
            * (stability ** -self.w[9])
            * (_exp(self.w[10] * (1.0 - r)) - 1.0)
            * hard_penalty
            * easy_bonus
        )
        return max(stability * (1.0 + alpha), STABILITY_MIN)

    def _stability_after_failure(self, difficulty: float, stability: float, r: float) -> float:
        s_forget = (
            self.w[11]
            * (difficulty ** -self.w[12])
            * (((stability + 1.0) ** self.w[13]) - 1.0)
            * _exp(self.w[14] * (1.0 - r))
        )
        # A lapse cannot increase durable stability.
        return max(min(s_forget, stability), STABILITY_MIN)


def _exp(x: float) -> float:
    import math

    return math.exp(x)


@dataclass
class CardMemory:
    """Running FSRS state + rated-review tally for one simulated card."""

    stability: float = 0.0
    difficulty: float = 0.0
    n_reviews: int = 0
    successes: int = 0
    has_state: bool = False
    last_review_day: float = 0.0

    def review(self, engine: FsrsEngine, rating: int, day: float) -> None:
        """Apply one rated review at absolute ``day`` (in days)."""
        if not self.has_state:
            self.stability = engine.init_stability(rating)
            self.difficulty = engine.init_difficulty(rating)
            self.has_state = True
        else:
            elapsed = max(day - self.last_review_day, 0.0)
            r = retrievability(self.stability, elapsed, engine.decay)
            self.difficulty = engine.next_difficulty(self.difficulty, rating)
            self.stability = engine.next_stability(self.difficulty, self.stability, r, rating)
        self.last_review_day = day
        self.n_reviews += 1
        if rating >= HARD:
            self.successes += 1

    def retrievability_at(self, day: float, decay: float = FSRS5_DEFAULT_DECAY) -> float:
        if not self.has_state:
            return 0.0
        return retrievability(self.stability, max(day - self.last_review_day, 0.0), decay)


def predict_memory_recall(card: CardMemory, exam_day: float, decay: float = FSRS5_DEFAULT_DECAY) -> float | None:
    """The app's memory-model prediction for one card, in [0, 1].

    Returns ``None`` when the display gate isn't met (no memory state, or fewer
    than ``MIN_MEMORY_SAMPLES`` rated reviews) - mirroring
    ``rslib/src/stats/memory.rs`` and ``card.rs``.
    """
    if not card.has_state or card.n_reviews < MIN_MEMORY_SAMPLES:
        return None

    p_fsrs = _clamp(card.retrievability_at(exam_day, decay), 0.0, 1.0)
    p_obs = _clamp(card.successes / card.n_reviews, 0.0, 1.0)
    n = float(card.n_reviews)
    w = n / (n + MIN_MEMORY_SAMPLES)
    return _clamp(w * p_obs + (1.0 - w) * p_fsrs, 0.0, 1.0)
