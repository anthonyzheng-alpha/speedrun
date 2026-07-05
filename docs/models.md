# Models

All models use the Wilson score interval to calculate the confidence interval for the metric.

## Memory model

The memory model measures the probability the user will remember a fact on a card on an exam. In the app, the user is free to set the date of their exam (if left unset, the app defaults to 30 days from the current date).

The model calculates the probability based on the following factors:
- FSRS retrievability
- Number of reviews
- Number of successful reviews
- Exam date

The code can be found in [memory_model.rs](../rslib/src/stats/memory.rs).

Cutoff: Needs ≥ 3 rated reviews and a valid FSRS memory state (FSRS must be turned on)

## Performance model

The performance model measures the probability they will correctly answer a new exam-style question. On the app, there is an overall performance metric, and four section-specific performance metrics.

The model calculates the probability as such:
- Average FSRS retrievability of reviewed cards (weighted 1x per reviewed card). In addition, accuracy percentage of practice exam (weighted 3x per question). Thus, formula is p = (W_CARD × sum_retrievability + W_EXAM × correct) / (W_CARD × reviewed_cards + W_EXAM × total_questions) for each of the four sections of the MCAT.

The code can be found in [metrics.rs](../rslib/src/stats/metrics.rs).

Cutoff: For each section, must have completed ≥ 10 practice-exam questions in a section. For overall, must have completed ≥ 10 practice-exam questions in all sections.

## Readiness model

The readiness model measures the probability the user will be ready for their exam. On the app, there is an overall readiness metric, and four section-specific readiness metrics.

The model calculates the probability as such:
- Using the same p formula as the performance model, for each section: r = 118 + 14 × p. For overall: r = 472 + 56 × p

The code can be found in [metrics.rs](../rslib/src/stats/metrics.rs).

Cutoff: Same as the performance model.