<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import Switch from "$lib/components/Switch.svelte";

    import {
        ALL_TOPICS,
        countAvailable,
        mergeBanks,
        TOPIC_LABELS,
        type QuestionBank,
        type TopicKey,
    } from "./lib";

    export let hardcoded: QuestionBank;
    export let generated: QuestionBank;
    export let onStart: (
        count: number,
        topics: TopicKey[],
        useGenerated: boolean,
    ) => void;

    const USE_GENERATED_KEY = "practiceExam.useGenerated";

    function loadUseGenerated(): boolean {
        if (typeof localStorage === "undefined") {
            return true;
        }
        return localStorage.getItem(USE_GENERATED_KEY) !== "false";
    }

    const MAX_QUESTIONS = 50;

    let requestedCount = 5;
    let useGenerated = loadUseGenerated();
    let enabled: Record<TopicKey, boolean> = {
        biology_biochemistry: true,
        chemistry_physics: true,
        psychology_sociology: true,
        cars: true,
    };

    $: if (typeof localStorage !== "undefined") {
        localStorage.setItem(USE_GENERATED_KEY, useGenerated ? "true" : "false");
    }

    $: enabledTopics = ALL_TOPICS.filter((t) => enabled[t]);
    // Questions available from the bundled banks (used directly when generation
    // is off, and as the fallback pool when it is on).
    $: bundledAvailable = countAvailable(
        mergeBanks(hardcoded, generated, false),
        enabledTopics,
    );
    // With live generation on, the count isn't bounded by the bundled banks.
    $: maxCount = useGenerated ? MAX_QUESTIONS : Math.max(1, bundledAvailable);
    $: effectiveCount = Math.min(Math.max(1, requestedCount || 0), maxCount);
    $: canStart =
        enabledTopics.length > 0 && (useGenerated || bundledAvailable > 0);

    function start() {
        if (!canStart) {
            return;
        }
        onStart(effectiveCount, enabledTopics, useGenerated);
    }
</script>

<div class="practice-exam">
    <h1>Practice Exam</h1>
    <div class="subtitle">
        Choose how many questions and which MCAT sections to cover.
    </div>

    <div class="pe-card">
        <div class="pe-count-row">
            <label for="pe-count">Number of questions</label>
            <input
                id="pe-count"
                type="number"
                min="1"
                max={maxCount}
                bind:value={requestedCount}
            />
        </div>
        <div class="subtitle" style="margin: 0;">
            {#if useGenerated}
                The exam will use {effectiveCount} freshly generated question{effectiveCount ===
                1
                    ? ""
                    : "s"}.
            {:else}
                {bundledAvailable} question{bundledAvailable === 1 ? "" : "s"} available
                for the selected sections. The exam will use {effectiveCount}.
            {/if}
        </div>
    </div>

    <div class="pe-card">
        <div class="pe-topic-row">
            <label for="pe-use-generated">Use AI-generated problems</label>
            <Switch id="pe-use-generated" bind:value={useGenerated} />
        </div>
        {#if !useGenerated}
            <div class="subtitle" style="margin: 0;">
                AI-generated problems are off. You'll see fewer, less varied questions
                drawn only from the hardcoded bank.
            </div>
        {:else}
            <div class="subtitle" style="margin: 0;">
                Fresh problems are generated with AI when you start - this may take a
                moment. Saved questions are used if generation is unavailable.
            </div>
        {/if}
    </div>

    <div class="pe-card">
        {#each ALL_TOPICS as topic (topic)}
            <div class="pe-topic-row">
                <label for={`pe-topic-${topic}`}>{TOPIC_LABELS[topic]}</label>
                <Switch id={`pe-topic-${topic}`} bind:value={enabled[topic]} />
            </div>
        {/each}
    </div>

    <div class="pe-actions">
        <button class="btn btn-primary" disabled={!canStart} on:click={start}>
            Start exam
        </button>
    </div>
</div>
