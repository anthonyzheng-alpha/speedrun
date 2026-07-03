# Speedrun

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
    - Hardcoded problems derived from Kaplan's MCAT test prep books.
    - AI-generated problems (see below). A toggle on the practice-exam config screen turns these on or off; when off, only the hardcoded problems are used and the user is warned of reduced problem diversity.

### AI-generated practice problems

When the "Use AI-generated problems" toggle is on, questions are generated **in real time** as the exam starts (a loading state is shown while this happens). Each generated question is verified before it is shown, and the app falls back to the bundled question banks if generation is unavailable or returns too few questions.

Grounding and verification:

- Questions are grounded on the [Jack Westin MCAT books](https://jackwestin.com/mcat-books) for each section (passed to the prompt as context). Sections can span multiple books:
    - Biology & Biochemistry -> biology, biochemistry
    - Chemistry & Physics -> general-chemistry, organic-chemistry, physics
    - Psychology & Sociology -> behavioral-sciences
    - CARS -> cars
- The eval gate is a blind independent solve: the model re-answers each generated question with no answer key, and only questions it re-derives correctly are kept. (Kaplan text is copyrighted and not fetched; it informs prompt style only.)
- Generated problems are categorized into the four MCAT sections, so answering them correctly/incorrectly feeds the same performance/readiness metrics as the hardcoded problems.

Where generation runs and how the key is supplied:

- **Desktop**: generation runs server-side in the local Python media server (endpoint `POST /_anki/generatePracticeExam`), so the key stays in the Python process. The app reads `OPENAI_API_KEY` from the environment, or from the repo-root `.env` file. Export it before launching, or keep it in `speedrun/.env`.
- **Android**: generation runs in Kotlin via OkHttp. The key is injected at build time into `BuildConfig.OPENAI_API_KEY` from `local.properties` (`OPENAI_API_KEY=...`), the `OPENAI_API_KEY` env var, or `speedrun-android/.env`. If no key is present, the app uses the bundled banks.
- The key is never committed (both `.env` files are gitignored). Note that embedding the key in an Android build makes it extractable from the APK, which is acceptable for a personal/dev build but not for public distribution.

## Mobile (Android)

The Android app lives in two sibling repositories (folder names are required by Gradle):

```
parent-folder/
  speedrun-android/          ← Kotlin UI (AnkiDroid fork)
  speedrun-android-backend/  ← MCAT Rust backend
```

- [speedrun-android](https://github.com/anthonyzheng-alpha/speedrun-android) — UI, practice exam, exam coverage
- [speedrun-android-backend](https://github.com/anthonyzheng-alpha/speedrun-android-backend) — Rust backend with MCAT scheduling and metrics

To run on an Android emulator, follow the **Run on an Android emulator** section in the [speedrun-android README](https://github.com/anthonyzheng-alpha/speedrun-android#run-on-an-android-emulator). You do not need this desktop repo to build the Android app.

## Getting Started

For more information on building, developing, please see [Development](./docs/development.md).

## Installer

Run `tools/build-installer` to build the installer.

Depending on your operating system, this produces a file under `out/installer/dist`:

- An MSI installer on Windows.
- A .dmg file on macOS.
- A tarball on Linux.
