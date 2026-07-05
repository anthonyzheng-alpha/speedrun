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
        generatePracticeExam,
        mergeBanks,
        scoreByTopic,
        type TopicKey,
    } from "./lib";
    import type { PageData } from "./$types";

    export let data: PageData;

    let phase: ExamPhase = "config";
    let items: ExamItem[] = [];
    let currentIndex = 0;
    let notice = "";

    async function start(count: number, topics: TopicKey[], useGenerated: boolean) {
        notice = "";
        if (!useGenerated) {
            beginExam(buildExam(mergeBanks(data.hardcoded, data.generated, false), count, topics));
            return;
        }

        phase = "loading";
        try {
            const live = await generatePracticeExam(count, topics);
            // Top up from the bundled banks if generation returned fewer than asked.
            let pool = live;
            if (live.length < count) {
                const haveIds = new Set(live.map((q) => q.id));
                const fallback = mergeBanks(data.hardcoded, data.generated, true).filter(
                    (q) => !haveIds.has(q.id),
                );
                pool = [...live, ...fallback];
            }
            beginExam(buildExam(pool, count, topics));
        } catch (error) {
            console.error("live generation failed, using bundled questions", error);
            notice =
                "Couldn't generate new questions right now - using saved questions instead.";
            beginExam(buildExam(mergeBanks(data.hardcoded, data.generated, true), count, topics));
        }
    }

    function beginExam(built: ExamItem[]) {
        items = built;
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

    async function next() {
        if (currentIndex < items.length - 1) {
            currentIndex += 1;
        } else {
            await persistResults();
            phase = "results";
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
            pycmd("refresh_home_metrics");
        } catch (error) {
            console.error("failed to record practice exam", error);
        }
    }

    function retake() {
        items = [];
        currentIndex = 0;
        notice = "";
        phase = "config";
    }
</script>

{#if phase === "config"}
    <ExamConfig hardcoded={data.hardcoded} generated={data.generated} onStart={start} />
{:else if phase === "loading"}
    <div class="practice-exam">
        <h1>Generating your exam...</h1>
        <div class="subtitle">
            Writing and checking fresh MCAT questions with AI. This can take a little
            while - hang tight.
        </div>
        <div class="pe-card">Please wait while we prepare your questions.</div>
    </div>
{:else if phase === "in-progress"}
    {#if notice}
        <div class="practice-exam" style="padding-bottom: 0;">
            <div class="subtitle">{notice}</div>
        </div>
    {/if}
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
