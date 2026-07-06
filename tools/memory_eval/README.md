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
   copied from `rslib/src/stats/memory.rs`): a multiplicative "encoding x
   retention" estimate - the observed success rate times the exam-date
   retrievability - gated at >= 3 rated reviews.
5. Has each student take several practice exams; each question is answered
   correctly with probability equal to the section's true recall.
6. Scores the predicted section recall against actual outcomes as a **Brier
   score** and renders a calibration (reliability) diagram plus a per-section
   Brier bar chart.

## Faithfulness

- Retrievability uses the FSRS-5 power forgetting curve with the same default
  decay (`-0.5`) as the Rust `fsrs` crate, so it matches
  `current_retrievability_seconds`.
- The prediction is copied verbatim from `memory.rs` (`p = p_obs * p_fsrs`).
  The previous weighted-average blend is kept as
  `fsrs_model.predict_memory_recall_blend` and is selectable via
  `--predict-mode blend` for before/after comparison.
- Stability/difficulty updates use the standard FSRS-5 equations with published
  default parameters. This validates the *algorithm*, not the compiled binary.

## Why multiplicative

An earlier version blended the two signals as a weighted *average*, which
averaged in the optimistic retrievability term and consistently
**over-predicted** recall (the reliability curve sat below the diagonal, mean
forecast ~0.13 above actual). Modelling recall as encoding strength (observed
hit rate) times retention (retrievability) removes that upward bias: in the
simulation the calibration bias drops from about +0.13 to near zero and the
Brier score falls to essentially the irreducible noise floor. Reproduce the old
behaviour with `--predict-mode blend`.

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
| `--predict-mode` | multiplicative | `multiplicative` (shipping) or `blend` (old weighted average) |
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
that the model's forecasts are positively correlated with observed recall, that
the multiplicative model is less over-confident than the old blend on a
deliberately mis-scheduled "crammer" cohort, and that its overall calibration
bias is smaller than the blend's.

## Practice-problems effectiveness experiment

`experiment.py` reuses the same synthetic-student machinery to answer a
different question: **does the AI practice-problems feature improve
performance?** It runs the three builds from the project `README.md` ("Study
feature testing") as arms of a controlled, paired experiment and reports the
**performance metric = accuracy on a final held-out MCAT exam** for each build.

```bash
cd speedrun/tools/memory_eval
python experiment.py --students 1000 --seed 0
```

The three arms:

- **Build 1** - 20 min studying (own or built-in deck) + an AI practice exam.
- **Build 2** - 20 min studying (own or built-in deck), no practice exam.
- **Build 3** - 20 min studying (built-in deck only), no practice exam.

How the 20-minute study block is modelled (the 20 min is simulated, not real
time): each student starts with a latent per-section mastery `m0` and each
retrieval event nudges it up with diminishing returns
(`m <- m + eta*(1 - m)`). Flashcards use `eta_card`; Build 1's practice
problems use `eta_exam = exam_boost * eta_card` (default `exam_boost = 3`,
matching `W_EXAM=3` vs `W_CARD=1` in `rslib/src/stats/metrics.rs`). Builds that
can make their own deck (1 and 2) spend study on their weak sections; the
built-in-only build (3) studies evenly. The boosted mastery feeds the same FSRS
card simulation, so forgetting before the exam still applies, and each
final-exam question is a Bernoulli draw at the resulting recall.

Outputs land in `out/`:

- `experiment_performance.png` - grouped bar chart of final-exam accuracy by
  build (overall + per section) with 95% Wilson confidence intervals.
- `experiment_results.csv` - one row per simulated final-exam question.

Pass `--report ../../practice_problems_experiment.txt` to also write the summary
table to the repo root. The *direction* and *mechanism* of the effects reflect
the app's design; the effect *magnitude* depends on the learning-rate knobs
(`--eta-card`, `--exam-boost`, `--cards-per-min`, `--practice-questions-per-section`),
all of which are CLI flags you can vary.

### Useful flags (experiment)

| Flag | Default | Meaning |
|------|---------|---------|
| `--students` | 1000 | Synthetic students per build |
| `--seed` | 0 | RNG seed (reproducible) |
| `--study-minutes` | 20 | Simulated study-block length |
| `--cards-per-min` | 3 | Flashcards reviewed per minute |
| `--eta-card` | 0.05 | Per-flashcard learning rate |
| `--exam-boost` | 3 | Practice-problem rate as a multiple of `--eta-card` |
| `--practice-questions-per-section` | 10 | Practice-exam questions in Build 1 |
| `--questions-per-section` | 20 | Final-exam questions per section |
| `--out` | `out` | Output directory |
| `--report` | (none) | Also write the text summary to this path |
