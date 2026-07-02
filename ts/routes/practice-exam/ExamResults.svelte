<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import {
        ALL_TOPICS,
        scoreByTopic,
        scoreExam,
        TOPIC_LABELS,
        type ExamItem,
    } from "./lib";

    export let items: ExamItem[];
    export let onRetake: () => void;

    const letters = ["A", "B", "C", "D", "E", "F"];

    $: correct = scoreExam(items);
    $: total = items.length;
    $: percent = total > 0 ? Math.round((correct / total) * 100) : 0;
    $: byTopic = scoreByTopic(items);
    $: usedTopics = ALL_TOPICS.filter((t) => byTopic[t].total > 0);
</script>

<div class="practice-exam">
    <h1>Results</h1>

    <div class="pe-card">
        <div class="pe-score">{correct} / {total} ({percent}%)</div>
        <div class="subtitle" style="margin: 0;">Correct answers</div>
    </div>

    <div class="pe-card">
        {#each usedTopics as topic (topic)}
            <div class="pe-breakdown-row">
                <span>{TOPIC_LABELS[topic]}</span>
                <span>{byTopic[topic].correct} / {byTopic[topic].total}</span>
            </div>
        {/each}
    </div>

    <h1 style="font-size: 1.15rem;">Review</h1>
    {#each items as item, i (item.question.id)}
        <div class="pe-card">
            <div class="pe-progress">
                Question {i + 1} &middot; {TOPIC_LABELS[item.question.topic]}
            </div>
            {#if item.question.passage}
                <div class="pe-passage">{item.question.passage}</div>
            {/if}
            <div class="pe-question-stem">{item.question.question}</div>

            {#each item.question.choices as choice, choiceIndex (choiceIndex)}
                <div
                    class="pe-review-choice"
                    class:correct={choiceIndex === item.question.answerIndex}
                    class:incorrect={choiceIndex === item.selectedIndex &&
                        item.selectedIndex !== item.question.answerIndex}
                >
                    <strong>{letters[choiceIndex]}.</strong>
                    {choice}
                    {#if choiceIndex === item.question.answerIndex}
                        <em>(correct answer)</em>
                    {:else if choiceIndex === item.selectedIndex}
                        <em>(your answer)</em>
                    {/if}
                </div>
            {/each}

            {#if item.selectedIndex === null}
                <div class="pe-explanation"><em>You did not answer this question.</em></div>
            {/if}
            {#if item.question.explanation}
                <div class="pe-explanation">{item.question.explanation}</div>
            {/if}
        </div>
    {/each}

    <div class="pe-actions">
        <button class="btn btn-primary" on:click={onRetake}>Retake</button>
    </div>
</div>
