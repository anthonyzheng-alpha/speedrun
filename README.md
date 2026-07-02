# Anki

[![Build Status](https://github.com/ankitects/anki/actions/workflows/ci.yml/badge.svg)](https://github.com/ankitects/anki/actions/workflows/ci.yml)
[![Documentation](https://img.shields.io/badge/docs-dev--docs.ankiweb.net-blue)](https://dev-docs.ankiweb.net)

This repo contains the source code for the computer version of an
[Anki](https://apps.ankiweb.net) clone that is meant for MCAT studying along with some additional features as outlined below.

## About

In addition to specializing in MCAT studying, this clone also implements:

- Topic-aware scheduling. The idea is that weaker topics are presented back to the user sooner without changing FSRS intervals. This setting can be toggled in the settings menu:
    - Deck > Options > Turn on Topic-Aware Scheduling > Scroll to Advanced.
- A memory model. For each card, this model determines the probability the user will remember the fact on an exam paired with a range and confidence percentage. If not enough data is available, the model will inform the user that it cannot make a conclusion at this time.
- A performance model. Located in the home screen, this model estimates the chance the user will correctly answer an exam-style question. If not enough data is available, the model will inform the user that it cannot make a conclusion at this time.
- A readiness model. Located in the home screen, this model estimates the approximate MCAT score the user would get based on their current performance in the app. If not enough data is available, the model will inform the user that it cannot make a conclusion at this time.
- Exam coverage progress. In the home menu, there are percentages that tell the user the percent of MCAT exam content they have mastered with the cards. There is an overall percentage as well as a breakdown by topic.
- Practice exam mode. This mode allows the user to take a practice exam with questions from the MCAT exam content and personalize the content to their needs.
    - Problems derived from Kaplan's MCAT test prep books.

There is also a mobile version found here: https://github.com/anthonyzheng-alpha/speedrun-mobile

## Getting Started

For more information on building and developing, please see [Development](./docs/development.md).
