// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import generatedBank from "./generated-questions.json";
import questionBank from "./questions.json";

import type { QuestionBank } from "./lib";
import type { PageLoad } from "./$types";

export const load = (async () => {
    return {
        hardcoded: questionBank as QuestionBank,
        generated: generatedBank as QuestionBank,
    };
}) satisfies PageLoad;
