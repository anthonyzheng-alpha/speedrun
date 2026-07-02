// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

export type TopicKey =
    | "biology_biochemistry"
    | "chemistry_physics"
    | "psychology_sociology"
    | "cars";

export interface PracticeQuestion {
    id: string;
    topic: TopicKey;
    passage?: string;
    question: string;
    choices: string[];
    answerIndex: number;
    explanation?: string;
}

export interface QuestionBank {
    topics: TopicKey[];
    questions: PracticeQuestion[];
}

export const ALL_TOPICS: TopicKey[] = [
    "biology_biochemistry",
    "chemistry_physics",
    "psychology_sociology",
    "cars",
];

export const TOPIC_LABELS: Record<TopicKey, string> = {
    biology_biochemistry: "Biology & Biochemistry",
    chemistry_physics: "Chemistry & Physics",
    psychology_sociology: "Psychology & Sociology",
    cars: "CARS",
};

/** One question plus the answer the user has (or hasn't yet) selected. */
export interface ExamItem {
    question: PracticeQuestion;
    selectedIndex: number | null;
}

/** The high level screen the exam UI is currently showing. */
export type ExamPhase = "config" | "in-progress" | "results";

/** How many questions are available for the currently-enabled topics. */
export function countAvailable(
    bank: QuestionBank,
    enabledTopics: TopicKey[],
): number {
    return bank.questions.filter((q) => enabledTopics.includes(q.topic)).length;
}

/** Fisher-Yates shuffle producing a new array (does not mutate input). */
function shuffle<T>(items: T[]): T[] {
    const result = items.slice();
    for (let i = result.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [result[i], result[j]] = [result[j], result[i]];
    }
    return result;
}

/**
 * Pick `count` random questions restricted to the enabled topics. If fewer
 * questions are available than requested, every available question is used.
 */
export function buildExam(
    bank: QuestionBank,
    count: number,
    enabledTopics: TopicKey[],
): ExamItem[] {
    const pool = bank.questions.filter((q) => enabledTopics.includes(q.topic));
    const chosen = shuffle(pool).slice(0, Math.max(0, count));
    return chosen.map((question) => ({ question, selectedIndex: null }));
}

/** Number of correctly answered items. */
export function scoreExam(items: ExamItem[]): number {
    return items.filter(
        (item) => item.selectedIndex === item.question.answerIndex,
    ).length;
}

/** Correct-answer counts grouped by topic, for the results breakdown. */
export function scoreByTopic(
    items: ExamItem[],
): Record<TopicKey, { correct: number; total: number }> {
    const result = {} as Record<TopicKey, { correct: number; total: number }>;
    for (const topic of ALL_TOPICS) {
        result[topic] = { correct: 0, total: 0 };
    }
    for (const item of items) {
        const bucket = result[item.question.topic];
        bucket.total += 1;
        if (item.selectedIndex === item.question.answerIndex) {
            bucket.correct += 1;
        }
    }
    return result;
}
