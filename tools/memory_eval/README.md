# Memory-model accuracy simulation

A standalone harness that estimates how well the per-card **memory model**
(predicted recall chance) is calibrated, using synthetic learners. It runs in
seconds with no backend build.

## What it does

1. Loads the real deck/exam structure: flashcards per MCAT section/subtopic from
   `rslib/src/collection/mcat.rs` and practice-question counts from
   `ts/routes/practice-exam/questions.json`.
2. Generates synthetic students, each with a latent per-section **mastery**
   (durable recall ceiling) kept separate from their **diligence** (how many
   spaced reviews, over how many days). This separation lets the model be wrong.
3. Simulates each student studying every card on backdated, spaced days. Review
   success is `mastery x FSRS retrievability`; success -> "Good", miss ->
   "Again", updating the FSRS state.
4. Predicts recall with the app's exact formula (`fsrs_model.predict_memory_recall`,
   copied from `rslib/src/stats/memory.rs`): blend the exam-date retrievability
   with the observed success rate, gated at >= 3 rated reviews.
5. Has each student take several practice exams; each question is answered
   correctly with probability equal to the section's true recall.
6. Scores the predicted section recall against actual outcomes as a **Brier
   score** and renders a calibration (reliability) diagram plus a per-section
   Brier bar chart.

## Faithfulness

- Retrievability uses the FSRS-5 power forgetting curve with the same default
  decay (`-0.5`) as the Rust `fsrs` crate, so it matches
  `current_retrievability_seconds`.
- The prediction blend is copied verbatim from `memory.rs`.
- Stability/difficulty updates use the standard FSRS-5 equations with published
  default parameters. This validates the *algorithm*, not the compiled binary.

## Run

```bash
cd speedrun/tools/memory_eval
pip install -r requirements.txt
python simulate.py --students 200 --seed 0
```

Outputs land in `out/`:

- `reliability.png` - calibration curve with the overall Brier score (the main graph)
- `brier_by_section.png` - Brier score per MCAT section
- `results.csv` - one row per simulated practice question, for your own analysis

## Useful flags

| Flag | Default | Meaning |
|------|---------|---------|
| `--students` | 200 | Number of synthetic learners |
| `--seed` | 0 | RNG seed (reproducible) |
| `--exams` | 5 | Practice exams per student |
| `--questions-per-section` | 6 | Questions per section per exam |
| `--exam-horizon-days` | -1 | Fixed days from study end to exam (`-1` = random per student) |
| `--card-sigma` | 0.10 | Per-card mastery noise |
| `--sections` | all | Comma-separated topic keys, e.g. `biology_biochemistry,cars` |
| `--include-generated` | off | Also count the generated question bank |
| `--control` | none | `perfect` sets forecast == truth (calibration floor) |
| `--out` | `out` | Output directory |

## Interpreting the Brier score

- Lower is better. `0.0` is perfect; `0.25` is a coin flip at base rate 0.5.
- Because exam outcomes are Bernoulli draws, there is an **irreducible floor**
  equal to the average `p*(1-p)`. Compare the model run against the
  `--control perfect` run: the gap is the model's own calibration error, and
  the floor is unavoidable noise.

## Sanity checks

```bash
python sanity.py
```

Verifies that the perfect-calibration control beats the model (lower Brier),
that the model's forecasts are positively correlated with observed recall, and
that a deliberately mis-scheduled "crammer" cohort shows over-confidence.
