# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Real-time MCAT practice-problem generation for the desktop app.

Called server-side from the media server so the OpenAI API key stays in the
Python process and is never exposed to the web frontend. Generation is followed
by a blind independent-solve verification step; only questions the model can
re-answer correctly are returned.
"""

from __future__ import annotations

import json
import math
import os
import random
from datetime import datetime
from pathlib import Path
from typing import Any

import requests

OPENAI_URL = "https://api.openai.com/v1/chat/completions"
MODEL = os.environ.get("OPENAI_MODEL", "gpt-4o")
REQUEST_TIMEOUT = 120

TOPICS = [
    "biology_biochemistry",
    "chemistry_physics",
    "psychology_sociology",
    "cars",
]

TOPIC_LABELS = {
    "biology_biochemistry": "Biology & Biochemistry",
    "chemistry_physics": "Chemistry & Physics",
    "psychology_sociology": "Psychology & Sociology",
    "cars": "CARS",
}

# Each MCAT section can be covered by several Jack Westin books.
TOPIC_BOOKS = {
    "biology_biochemistry": ["biology", "biochemistry"],
    "chemistry_physics": ["general-chemistry", "organic-chemistry", "physics"],
    "psychology_sociology": ["behavioral-sciences"],
    "cars": ["cars"],
}

TOPIC_ID_PREFIX = {
    "biology_biochemistry": "live-bb",
    "chemistry_physics": "live-cp",
    "psychology_sociology": "live-ps",
    "cars": "live-cars",
}


class PracticeExamGenError(Exception):
    """Raised when generation cannot proceed (e.g. missing key or API error)."""


def resolve_api_key() -> str:
    """Read OPENAI_API_KEY from the environment, else from a repo-root .env."""
    key = os.environ.get("OPENAI_API_KEY")
    if key:
        return key.strip()
    # Python does not auto-load .env; look in the cwd and the repo root
    # (qt/aqt/ -> speedrun/).
    candidates = [Path.cwd() / ".env", Path(__file__).resolve().parents[2] / ".env"]
    for env_path in candidates:
        try:
            if not env_path.exists():
                continue
            for line in env_path.read_text(encoding="utf-8").splitlines():
                line = line.strip()
                if line.startswith("OPENAI_API_KEY="):
                    value = line.split("=", 1)[1].strip().strip('"').strip("'")
                    if value:
                        return value
        except OSError:
            continue
    raise PracticeExamGenError(
        "OPENAI_API_KEY is not set. Export it or add it to the repo .env file."
    )


def _chat_json(
    api_key: str, messages: list[dict[str, str]], temperature: float
) -> dict[str, Any]:
    try:
        resp = requests.post(
            OPENAI_URL,
            headers={
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json",
            },
            json={
                "model": MODEL,
                "temperature": temperature,
                "response_format": {"type": "json_object"},
                "messages": messages,
            },
            timeout=REQUEST_TIMEOUT,
        )
        resp.raise_for_status()
    except requests.RequestException as exc:
        raise PracticeExamGenError(f"OpenAI request failed: {exc}") from exc
    content = resp.json()["choices"][0]["message"]["content"]
    return json.loads(content)


def _valid(problem: dict[str, Any], topic: str) -> bool:
    if not isinstance(problem, dict):
        return False
    if not isinstance(problem.get("question"), str) or len(problem["question"].strip()) < 10:
        return False
    choices = problem.get("choices")
    if not isinstance(choices, list) or len(choices) != 4:
        return False
    if any(not isinstance(c, str) or not c.strip() for c in choices):
        return False
    if len({c.strip().lower() for c in choices}) != 4:
        return False
    idx = problem.get("answerIndex")
    if not isinstance(idx, int) or idx < 0 or idx > 3:
        return False
    if problem.get("topic") != topic:
        return False
    return True


def _generation_messages(topic: str, count: int) -> list[dict[str, str]]:
    books = ", ".join(TOPIC_BOOKS.get(topic, []))
    is_cars = topic == "cars"
    if is_cars:
        style = (
            "This is CARS (Critical Analysis and Reasoning): each problem MUST include a "
            "self-contained `passage` field (a 250-350 word humanities/social-science passage) "
            "and a question answerable SOLELY from that passage. Do not test outside knowledge."
        )
    else:
        style = (
            "Each problem tests MCAT-level science knowledge. Leave `passage` empty unless a "
            "short data/experiment stem is needed."
        )
    book_note = (
        f"Draw content from the Jack Westin MCAT books for this section: {books}.\n"
        if books
        else ""
    )
    return [
        {
            "role": "system",
            "content": (
                "You are an expert MCAT item writer creating rigorous, exam-accurate "
                "multiple-choice questions in the style of Kaplan and Jack Westin MCAT prep "
                "books. Every question must have exactly four choices with a single unambiguous "
                "best answer and three plausible distractors. Return ONLY JSON."
            ),
        },
        {
            "role": "user",
            "content": (
                f'Write {count} NEW MCAT-style multiple-choice questions for the topic '
                f'"{TOPIC_LABELS[topic]}".\n{book_note}{style}\n\n'
                'Return a JSON object of the form:\n'
                '{ "problems": [ { "topic": "' + topic + '", "passage": "", "question": "...", '
                '"choices": ["...","...","...","..."], "answerIndex": 0, "explanation": "..." } ] }\n'
                f'Every problem must set "topic" to exactly "{topic}", have 4 distinct choices, '
                'an integer "answerIndex" between 0 and 3, and a concise explanation.'
            ),
        },
    ]


def _generate_candidates(api_key: str, topic: str, count: int) -> list[dict[str, Any]]:
    data = _chat_json(api_key, _generation_messages(topic, count), 0.8)
    problems = data.get("problems", [])
    if not isinstance(problems, list):
        return []
    out = []
    for problem in problems:
        if not isinstance(problem, dict):
            continue
        problem["topic"] = topic
        problem.setdefault("passage", "")
        problem.setdefault("explanation", "")
        if _valid(problem, topic):
            out.append(problem)
    return out


def _verify(api_key: str, candidates: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Blind independent solve: keep only questions the model re-answers correctly."""
    if not candidates:
        return []
    items = [
        {
            "index": i,
            "passage": c.get("passage", ""),
            "question": c["question"],
            "choices": c["choices"],
        }
        for i, c in enumerate(candidates)
    ]
    messages = [
        {
            "role": "system",
            "content": (
                "You are an MCAT expert taking a multiple-choice exam. For each item choose the "
                "single best answer using only the passage (if any) and established knowledge. "
                'Respond ONLY with JSON: {"answers": [{"index": <int>, "answerIndex": <0-3>}]}.'
            ),
        },
        {
            "role": "user",
            "content": json.dumps({"items": items}),
        },
    ]
    data = _chat_json(api_key, messages, 0)
    answers = {}
    for entry in data.get("answers", []):
        if isinstance(entry, dict) and "index" in entry:
            answers[entry["index"]] = entry.get("answerIndex")
    return [c for i, c in enumerate(candidates) if answers.get(i) == c["answerIndex"]]


EVAL_LOG_NAME = "practice_exam_eval.txt"
_STATE_PREFIX = "#STATE "


def _eval_log_path() -> Path:
    """Location of the accuracy log; overridable via PRACTICE_EXAM_EVAL_LOG."""
    override = os.environ.get("PRACTICE_EXAM_EVAL_LOG")
    if override:
        return Path(override)
    # qt/aqt/ -> speedrun/ (repo root), matching the .env resolution above.
    return Path(__file__).resolve().parents[2] / EVAL_LOG_NAME


def _read_last_state(path: Path) -> dict[str, Any]:
    """Return the cumulative totals from the last #STATE line, or empty."""
    try:
        if not path.exists():
            return {}
        last: dict[str, Any] = {}
        for line in path.read_text(encoding="utf-8").splitlines():
            if line.startswith(_STATE_PREFIX):
                try:
                    last = json.loads(line[len(_STATE_PREFIX):])
                except json.JSONDecodeError:
                    continue
        return last if isinstance(last, dict) else {}
    except OSError:
        return {}


def _fmt_topics(tallies: dict[str, list[int]]) -> str:
    parts = []
    for topic in sorted(tallies):
        verified, total = tallies[topic]
        pct = (100.0 * verified / total) if total else 0.0
        parts.append(f"{topic} {verified}/{total} ({pct:.1f}%)")
    return ", ".join(parts)


def _log_eval(
    candidates: list[dict[str, Any]], vetted: list[dict[str, Any]]
) -> None:
    """Append per-run and cumulative verify accuracy to the eval log."""
    if not candidates:
        return
    try:
        run: dict[str, list[int]] = {}
        for c in candidates:
            topic = c.get("topic", "unknown")
            run.setdefault(topic, [0, 0])[1] += 1
        for v in vetted:
            topic = v.get("topic", "unknown")
            run.setdefault(topic, [0, 0])[0] += 1

        run_total = sum(t[1] for t in run.values())
        run_verified = sum(t[0] for t in run.values())

        path = _eval_log_path()
        prior = _read_last_state(path)
        cum_topics: dict[str, list[int]] = {
            k: [int(v[0]), int(v[1])]
            for k, v in (prior.get("topics") or {}).items()
            if isinstance(v, (list, tuple)) and len(v) == 2
        }
        for topic, (verified, total) in run.items():
            cum = cum_topics.setdefault(topic, [0, 0])
            cum[0] += verified
            cum[1] += total
        cum_total = sum(t[1] for t in cum_topics.values())
        cum_verified = sum(t[0] for t in cum_topics.values())

        ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        run_pct = (100.0 * run_verified / run_total) if run_total else 0.0
        cum_pct = (100.0 * cum_verified / cum_total) if cum_total else 0.0
        state = {
            "candidates": cum_total,
            "verified": cum_verified,
            "topics": cum_topics,
        }
        block = (
            f"[{ts}] run: candidates={run_total} verified={run_verified} "
            f"accuracy={run_pct:.1f}% | {_fmt_topics(run)}\n"
            f"[{ts}] cumulative: candidates={cum_total} verified={cum_verified} "
            f"accuracy={cum_pct:.1f}% | {_fmt_topics(cum_topics)}\n"
            f"{_STATE_PREFIX}{json.dumps(state, separators=(',', ':'))}\n"
        )
        with path.open("a", encoding="utf-8") as fh:
            fh.write(block)
    except Exception as exc:  # noqa: BLE001 - logging must never break generation
        print(f"practice-exam eval logging failed: {exc}")


def generate_exam(count: int, topics: list[str]) -> list[dict[str, Any]]:
    """Generate and verify up to `count` questions across the enabled topics."""
    api_key = resolve_api_key()
    topics = [t for t in topics if t in TOPICS] or list(TOPICS)
    count = max(1, int(count))

    per_topic = max(1, math.ceil(count / len(topics)))
    candidates: list[dict[str, Any]] = []
    for topic in topics:
        candidates += _generate_candidates(
            api_key, topic, math.ceil(per_topic * 1.5)
        )

    vetted = _verify(api_key, candidates)
    _log_eval(candidates, vetted)
    random.shuffle(vetted)

    result: list[dict[str, Any]] = []
    counters: dict[str, int] = {}
    for problem in vetted[:count]:
        topic = problem["topic"]
        counters[topic] = counters.get(topic, 0) + 1
        result.append(
            {
                "id": f"{TOPIC_ID_PREFIX[topic]}-{counters[topic]:03d}",
                "topic": topic,
                "passage": problem.get("passage", ""),
                "question": problem["question"],
                "choices": problem["choices"],
                "answerIndex": problem["answerIndex"],
                "explanation": problem.get("explanation", ""),
            }
        )
    return result
