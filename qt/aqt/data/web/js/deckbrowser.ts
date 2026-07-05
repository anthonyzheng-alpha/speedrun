/* Copyright: Ankitects Pty Ltd and contributors
 * License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html */

$(init);

function init() {
    $("tr.deck").draggable({
        scroll: false,

        // can't use "helper: 'clone'" because of a bug in jQuery 1.5
        helper: function(_event) {
            return $(this).clone(false);
        },
        delay: 200,
        opacity: 0.7,
    });
    $("tr.deck").droppable({
        drop: handleDropEvent,
        hoverClass: "drag-hover",
    });
    $("tr.top-level-drag-row").droppable({
        drop: handleDropEvent,
        hoverClass: "drag-hover",
    });
}

function handleDropEvent(event, ui) {
    const draggedDeckId = ui.draggable.attr("id");
    const ontoDeckId = $(this).attr("id") || "";

    pycmd("drag:" + draggedDeckId + "," + ontoDeckId);
}

interface OnboardingStep {
    sel: string;
    title: string;
    body: string;
}

// One-time guided tour for first-time users. Highlights the Study, Practice,
// and Metrics areas of the deck browser. Invoked from the HTML rendered by
// deckbrowser.py when the "mcatOnboardingShown" config flag is not yet set.
(window as any).ankiStartOnboarding = function(steps: OnboardingStep[]) {
    const overlay = document.getElementById("onboardingOverlay");
    const spotlight = document.getElementById("onboardingSpotlight");
    const tooltip = document.getElementById("onboardingTooltip");
    const titleEl = document.getElementById("onboardingTitle");
    const textEl = document.getElementById("onboardingText");
    const progressEl = document.getElementById("onboardingProgress");
    const nextBtn = document.getElementById("onboardingNext");
    const skipBtn = document.getElementById("onboardingSkip");
    if (
        !overlay || !spotlight || !tooltip || !titleEl || !textEl
        || !progressEl || !nextBtn || !skipBtn
    ) {
        return;
    }

    // Only keep steps whose target element is actually present.
    const active = steps.filter((s) => document.querySelector(s.sel));

    function finish() {
        overlay!.style.display = "none";
        try {
            pycmd("onboarding_done");
        } catch (e) {
            // ignore
        }
    }

    if (active.length === 0) {
        finish();
        return;
    }

    let index = 0;
    const pad = 8;

    function position(step: OnboardingStep, i: number, target: Element) {
        const rect = target.getBoundingClientRect();
        spotlight!.style.top = rect.top - pad + "px";
        spotlight!.style.left = rect.left - pad + "px";
        spotlight!.style.width = rect.width + pad * 2 + "px";
        spotlight!.style.height = rect.height + pad * 2 + "px";

        titleEl!.textContent = step.title;
        textEl!.textContent = step.body;
        progressEl!.textContent = i + 1 + " / " + active.length;
        nextBtn!.textContent = i === active.length - 1 ? "Done" : "Next";

        const ttRect = tooltip!.getBoundingClientRect();
        let ttTop = rect.top + rect.height + pad + 12;
        if (ttTop + ttRect.height > window.innerHeight) {
            ttTop = rect.top - ttRect.height - pad - 12;
        }
        if (ttTop < 8) {
            ttTop = 8;
        }
        let ttLeft = rect.left - pad;
        const maxLeft = window.innerWidth - ttRect.width - 8;
        if (ttLeft > maxLeft) {
            ttLeft = maxLeft;
        }
        if (ttLeft < 8) {
            ttLeft = 8;
        }
        tooltip!.style.top = ttTop + "px";
        tooltip!.style.left = ttLeft + "px";
    }

    function show(i: number) {
        const step = active[i];
        const target = document.querySelector(step.sel);
        if (!target) {
            return;
        }
        overlay!.style.display = "block";
        target.scrollIntoView({ block: "center" });
        requestAnimationFrame(() => position(step, i, target));
    }

    nextBtn.addEventListener("click", () => {
        index += 1;
        if (index >= active.length) {
            finish();
        } else {
            show(index);
        }
    });
    skipBtn.addEventListener("click", (e) => {
        e.preventDefault();
        finish();
    });

    show(0);
};
