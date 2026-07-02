<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import "./practice-exam-base.scss";

    import { recordPracticeExam } from "@generated/backend";

    import ExamConfig from "./ExamConfig.svelte";
    import ExamQuestion from "./ExamQuestion.svelte";
    import ExamResults from "./ExamResults.svelte";
    import {
        buildExam,
        type ExamItem,
        type ExamPhase,
        scoreByTopic,
        type TopicKey,
    } from "./lib";
    import type { PageData } from "./$types";

    export let data: PageData;

    let phase: ExamPhase = "config";
    let items: ExamItem[] = [];
    let currentIndex = 0;

    function start(count: number, topics: TopicKey[]) {
        items = buildExam(data.bank, count, topics);
        currentIndex = 0;
        phase = items.length > 0 ? "in-progress" : "config";
    }

    function select(choiceIndex: number) {
        items[currentIndex].selectedIndex = choiceIndex;
        items = items;
    }

    function prev() {
        if (currentIndex > 0) {
            currentIndex -= 1;
        }
    }

    function next() {
        if (currentIndex < items.length - 1) {
            currentIndex += 1;
        } else {
            phase = "results";
            void persistResults();
        }
    }

    /** Save the completed exam so it feeds the performance/readiness metrics. */
    async function persistResults() {
        const byTopic = scoreByTopic(items);
        const results = (
            Object.entries(byTopic) as [TopicKey, { correct: number; total: number }][]
        )
            .filter(([, tally]) => tally.total > 0)
            .map(([topic, tally]) => ({
                topic,
                correct: tally.correct,
                total: tally.total,
            }));
        if (results.length === 0) {
            return;
        }
        try {
            await recordPracticeExam({ results });
        } catch (error) {
            console.error("failed to record practice exam", error);
        }
    }

    function retake() {
        items = [];
        currentIndex = 0;
        phase = "config";
    }
</script>

{#if phase === "config"}
    <ExamConfig bank={data.bank} onStart={start} />
{:else if phase === "in-progress"}
    <ExamQuestion
        item={items[currentIndex]}
        index={currentIndex}
        total={items.length}
        onSelect={select}
        onPrev={prev}
        onNext={next}
    />
{:else}
    <ExamResults {items} onRetake={retake} />
{/if}
