<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { TOPIC_LABELS, type ExamItem } from "./lib";

    export let item: ExamItem;
    export let index: number;
    export let total: number;
    export let onSelect: (choiceIndex: number) => void;
    export let onPrev: () => void;
    export let onNext: () => void;

    const letters = ["A", "B", "C", "D", "E", "F"];

    $: isLast = index === total - 1;
</script>

<div class="practice-exam">
    <div class="pe-progress">
        Question {index + 1} of {total} &middot; {TOPIC_LABELS[item.question.topic]}
    </div>

    {#if item.question.passage}
        <div class="pe-passage">{item.question.passage}</div>
    {/if}

    <div class="pe-question-stem">{item.question.question}</div>

    {#each item.question.choices as choice, choiceIndex (choiceIndex)}
        <button
            class="pe-choice"
            class:selected={item.selectedIndex === choiceIndex}
            on:click={() => onSelect(choiceIndex)}
        >
            <span class="pe-choice-letter">{letters[choiceIndex]}.</span>
            <span>{choice}</span>
        </button>
    {/each}

    <div class="pe-actions">
        <button class="btn btn-outline-secondary" disabled={index === 0} on:click={onPrev}>
            Back
        </button>
        <button class="btn btn-primary" on:click={onNext}>
            {isLast ? "Finish" : "Next"}
        </button>
    </div>
</div>
