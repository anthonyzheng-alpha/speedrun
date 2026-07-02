<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import Switch from "$lib/components/Switch.svelte";

    import {
        ALL_TOPICS,
        countAvailable,
        TOPIC_LABELS,
        type QuestionBank,
        type TopicKey,
    } from "./lib";

    export let bank: QuestionBank;
    export let onStart: (count: number, topics: TopicKey[]) => void;

    let requestedCount = 5;
    let enabled: Record<TopicKey, boolean> = {
        biology_biochemistry: true,
        chemistry_physics: true,
        psychology_sociology: true,
        cars: true,
    };

    $: enabledTopics = ALL_TOPICS.filter((t) => enabled[t]);
    $: available = countAvailable(bank, enabledTopics);
    $: effectiveCount = Math.min(Math.max(1, requestedCount || 0), available);
    $: canStart = enabledTopics.length > 0 && available > 0;

    function start() {
        if (!canStart) {
            return;
        }
        onStart(effectiveCount, enabledTopics);
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
                max={Math.max(1, available)}
                bind:value={requestedCount}
            />
        </div>
        <div class="subtitle" style="margin: 0;">
            {available} question{available === 1 ? "" : "s"} available for the selected
            sections. The exam will use {effectiveCount}.
        </div>
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
