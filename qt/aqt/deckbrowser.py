# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

import html
from copy import deepcopy
from dataclasses import dataclass
from typing import Any

import aqt
import aqt.operations
from anki import stats_pb2
from anki.collection import Collection, OpChanges
from anki.decks import DeckCollapseScope, DeckId, DeckTreeNode
from aqt import AnkiQt, gui_hooks
from aqt.deckoptions import display_options_for_deck_id
from aqt.operations import QueryOp
from aqt.operations.deck import (
    add_deck_dialog,
    remove_decks,
    rename_deck,
    reparent_decks,
    set_current_deck,
    set_deck_collapsed,
)
from aqt.qt import *
from aqt.sound import av_player
from aqt.toolbar import BottomBar
from aqt.utils import getOnlyText, openLink, shortcut, showInfo, tr


class DeckBrowserBottomBar:
    def __init__(self, deck_browser: DeckBrowser) -> None:
        self.deck_browser = deck_browser


@dataclass
class RenderData:
    """Data from collection that is required to show the page."""

    tree: DeckTreeNode
    current_deck_id: DeckId
    studied_today: str
    sched_upgrade_required: bool
    exam_coverage: stats_pb2.ExamCoverageResponse
    exam_metrics: stats_pb2.ExamMetricsResponse


@dataclass
class DeckBrowserContent:
    """Stores sections of HTML content that the deck browser will be
    populated with.

    Attributes:
        tree {str} -- HTML of the deck tree section
        stats {str} -- HTML of the stats section
    """

    tree: str
    stats: str


@dataclass
class RenderDeckNodeContext:
    current_deck_id: DeckId


class DeckBrowser:
    _render_data: RenderData

    def __init__(self, mw: AnkiQt) -> None:
        self.mw = mw
        self.web = mw.web
        self.bottom = BottomBar(mw, mw.bottomWeb)
        self.scrollPos = QPoint(0, 0)
        self._refresh_needed = False

    def show(self) -> None:
        av_player.stop_and_clear_queue()
        self.web.set_bridge_command(self._linkHandler, self)
        # redraw top bar for theme change
        self.mw.toolbar.redraw()
        self.refresh()

    def refresh(self) -> None:
        self._renderPage()
        self._refresh_needed = False

    def refresh_if_needed(self) -> None:
        if self._refresh_needed:
            self.refresh()

    def op_executed(
        self, changes: OpChanges, handler: object | None, focused: bool
    ) -> bool:
        if changes.study_queues and handler is not self:
            self._refresh_needed = True

        if focused:
            self.refresh_if_needed()

        return self._refresh_needed

    # Event handlers
    ##########################################################################

    def _linkHandler(self, url: str) -> Any:
        if ":" in url:
            (cmd, arg) = url.split(":", 1)
        else:
            cmd = url
            arg = ""
        if cmd == "open":
            self.set_current_deck(DeckId(int(arg)))
        elif cmd == "opts":
            self._showOptions(arg)
        elif cmd == "shared":
            self._onShared()
        elif cmd == "import":
            self.mw.onImport()
        elif cmd == "create":
            self._on_create()
        elif cmd == "practice_exam":
            self._on_practice_exam()
        elif cmd == "set_exam_date":
            self._on_set_exam_date()
        elif cmd == "drag":
            source, target = arg.split(",")
            self._handle_drag_and_drop(DeckId(int(source)), DeckId(int(target or 0)))
        elif cmd == "collapse":
            self._collapse(DeckId(int(arg)))
        elif cmd == "v2upgrade":
            self._confirm_upgrade()
        elif cmd == "v2upgradeinfo":
            if self.mw.col.sched_ver() == 1:
                openLink("https://faqs.ankiweb.net/the-anki-2.1-scheduler.html")
            else:
                openLink("https://faqs.ankiweb.net/the-2021-scheduler.html")
        elif cmd == "select":
            set_current_deck(
                parent=self.mw, deck_id=DeckId(int(arg))
            ).run_in_background()
        return False

    def set_current_deck(self, deck_id: DeckId) -> None:
        set_current_deck(parent=self.mw, deck_id=deck_id).success(
            lambda _: self.mw.onOverview()
        ).run_in_background(initiator=self)

    # HTML generation
    ##########################################################################

    _body = """
<center>
<table cellspacing=0 cellpadding=3>
%(tree)s
</table>

<br>
%(stats)s
</center>
"""

    def _renderPage(self, reuse: bool = False) -> None:
        if not reuse:

            def get_data(col: Collection) -> RenderData:
                return RenderData(
                    tree=col.sched.deck_due_tree(),
                    current_deck_id=col.decks.get_current_id(),
                    studied_today=col.studied_today(),
                    sched_upgrade_required=not col.v3_scheduler(),
                    exam_coverage=col.exam_coverage(),
                    exam_metrics=col.exam_metrics(),
                )

            def success(output: RenderData) -> None:
                self._render_data = output
                self.__renderPage(None)

            QueryOp(
                parent=self.mw,
                op=get_data,
                success=success,
            ).run_in_background()
        else:
            self.web.evalWithCallback("window.pageYOffset", self.__renderPage)

    def __renderPage(self, offset: int | None) -> None:
        data = self._render_data
        content = DeckBrowserContent(
            tree=self._renderDeckTree(data.tree),
            stats=self._renderStats(),
        )
        gui_hooks.deck_browser_will_render_content(self, content)
        self.web.stdHtml(
            self._v1_upgrade_message(data.sched_upgrade_required)
            + self._body % content.__dict__,
            css=["css/deckbrowser.css"],
            js=[
                "js/vendor/jquery.min.js",
                "js/vendor/jquery-ui.min.js",
                "js/deckbrowser.js",
            ],
            context=self,
        )
        self._drawButtons()
        if offset is not None:
            self._scrollToOffset(offset)
        gui_hooks.deck_browser_did_render(self)

    def _scrollToOffset(self, offset: int) -> None:
        self.web.eval("window.scrollTo(0, %d, 'instant');" % offset)

    def _renderStats(self) -> str:
        return (
            self._render_exam_coverage()
            + self._render_exam_metrics()
            + (
                '<div id="studiedToday"><span>{}</span></div>'.format(
                    self._render_data.studied_today
                )
            )
        )

    def _render_exam_coverage(self) -> str:
        """Show the percent of the MCAT exam studied so far, broken down by
        section. Hidden when there are no MCAT topics (e.g. a non-MCAT
        collection), so ordinary Anki users are unaffected."""
        coverage = self._render_data.exam_coverage
        if not coverage.topics_total:
            return ""

        sections = "".join(
            "<span class=exam-coverage-section>{name}: "
            "<b>{percent:.0f}%</b></span>".format(
                name=html.escape(section.section),
                percent=section.percent,
            )
            for section in coverage.sections
            if section.topics_total
        )
        note = (
            "A section's topics count as studied once every card in them is "
            "mastered (reviewed out to at least a 21-day interval). This is the "
            "share of topics covered in each section."
        )
        return """
<div id=examCoverage>
  <div class=exam-coverage-headline>MCAT exam studied: <b>{overall:.0f}%</b></div>
  <div class=exam-coverage-sections>{sections}</div>
  <div class=exam-coverage-note>{note}</div>
</div>
""".format(
            overall=coverage.overall_percent,
            sections=sections,
            note=note,
        )

    def _render_exam_metrics(self) -> str:
        """Show global performance (chance of answering a new exam-style
        question correctly) and readiness (projected MCAT score), overall and
        per section. Shown only for MCAT collections, like exam coverage."""
        if not self._render_data.exam_coverage.topics_total:
            return ""

        metrics = self._render_data.exam_metrics
        performance = metrics.performance_overall
        readiness = metrics.readiness_overall

        exam_date_row = self._exam_date_row()

        if not performance.has_enough_data:
            return (
                '<div id=examMetrics><hr class=exam-metrics-divider>'
                '{exam_date_row}'
                '<div class=exam-metrics-note>{msg}</div></div>'.format(
                    exam_date_row=exam_date_row,
                    msg=html.escape(performance.justification),
                )
            )

        performance_row = (
            "Performance: <b>{score:.0f}%</b> "
            "<span class=exam-metrics-envelope>({low:.0f}%\u2013{high:.0f}%, "
            "{conf:.0f}% confidence)</span>"
        ).format(
            score=performance.score,
            low=performance.range_min,
            high=performance.range_max,
            conf=performance.confidence,
        )
        readiness_row = (
            "Projected MCAT: <b>{rscore:.0f}</b> "
            "<span class=exam-metrics-envelope>({rlow:.0f}\u2013{rhigh:.0f})</span>"
        ).format(
            rscore=readiness.score,
            rlow=readiness.range_min,
            rhigh=readiness.range_max,
        )
        sections = "".join(
            "<span class=exam-metrics-section>{name}: <b>{score:.0f}%</b></span>".format(
                name=html.escape(section.section),
                score=section.estimate.score,
            )
            for section in metrics.performance_sections
            if section.estimate.has_enough_data
        )
        performance_note = (
            "Your estimated chance of answering a new exam-style question "
            "correctly. Based on your flashcard reviews and practice exams "
            "(practice exams count more)."
        )
        readiness_note = "Your projected MCAT score, derived from your section performance."
        return """
<div id=examMetrics>
  <hr class=exam-metrics-divider>
  {exam_date_row}
  <div class=exam-metrics-group>
    <div class=exam-metrics-headline>{performance_row}</div>
    <div class=exam-metrics-sections>{sections}</div>
    <div class=exam-metrics-note>{performance_note}</div>
  </div>
  <hr class=exam-metrics-divider>
  <div class=exam-metrics-group>
    <div class=exam-metrics-headline>{readiness_row}</div>
    <div class=exam-metrics-note>{readiness_note}</div>
  </div>
</div>
""".format(
            exam_date_row=exam_date_row,
            performance_row=performance_row,
            sections=sections,
            performance_note=performance_note,
            readiness_row=readiness_row,
            readiness_note=readiness_note,
        )

    def _exam_date_row(self) -> str:
        """A line showing the target exam date, with a link to change it. The
        memory and performance metrics are projected to this date."""
        import time

        exam_date_secs = self.mw.col.get_config("mcatExamDate", None)
        if exam_date_secs:
            date_str = time.strftime("%Y-%m-%d", time.localtime(int(exam_date_secs)))
            label = f"Exam date: <b>{html.escape(date_str)}</b>"
        else:
            label = "Exam date: <b>not set</b> (projecting to a 30-day horizon)"
        return (
            '<div class=exam-metrics-note>{label} '
            '<a href=# onclick="return pycmd(\'set_exam_date\')">change</a></div>'
        ).format(label=label)

    def _renderDeckTree(self, top: DeckTreeNode) -> str:
        buf = """
<tr><th colspan=5 align=start>{}</th>
<th class=count>{}</th>
<th class=count>{}</th>
<th class=count>{}</th>
<th class=optscol></th></tr>""".format(
            tr.decks_deck(),
            tr.actions_new(),
            tr.decks_learn_header(),
            tr.decks_review_header(),
        )
        buf += self._topLevelDragRow()

        ctx = RenderDeckNodeContext(current_deck_id=self._render_data.current_deck_id)

        for child in top.children:
            buf += self._render_deck_node(child, ctx)

        return buf

    def _render_deck_node(self, node: DeckTreeNode, ctx: RenderDeckNodeContext) -> str:
        if node.collapsed:
            prefix = "+"
        else:
            prefix = "−"

        def indent() -> str:
            return "&nbsp;" * 6 * (node.level - 1)

        if node.deck_id == ctx.current_deck_id:
            klass = "deck current"
        else:
            klass = "deck"

        buf = (
            "<tr class='%s' id='%d' onclick='if(event.shiftKey) return pycmd(\"select:%d\")'>"
            % (
                klass,
                node.deck_id,
                node.deck_id,
            )
        )
        # deck link
        if node.children:
            collapse = (
                "<a class=collapse href=# onclick='return pycmd(\"collapse:%d\")'>%s</a>"
                % (node.deck_id, prefix)
            )
        else:
            collapse = "<span class=collapse></span>"
        if node.filtered:
            extraclass = "filtered"
        else:
            extraclass = ""
        buf += """

        <td class=decktd colspan=5>%s%s<a class="deck %s"
        href=# onclick="return pycmd('open:%d')">%s</a></td>""" % (
            indent(),
            collapse,
            extraclass,
            node.deck_id,
            html.escape(node.name),
        )

        # due counts
        def nonzeroColour(cnt: int, klass: str) -> str:
            if not cnt:
                klass = "zero-count"
            return f'<span class="{klass}">{cnt}</span>'

        review = nonzeroColour(node.review_count, "review-count")
        learn = nonzeroColour(node.learn_count, "learn-count")

        buf += ("<td align=end>%s</td>" * 3) % (
            nonzeroColour(node.new_count, "new-count"),
            learn,
            review,
        )
        # options
        buf += (
            "<td align=center class=opts><a onclick='return pycmd(\"opts:%d\");'>"
            "<img src='/_anki/imgs/gears.svg' class=gears></a></td></tr>" % node.deck_id
        )
        # children
        if not node.collapsed:
            for child in node.children:
                buf += self._render_deck_node(child, ctx)
        return buf

    def _topLevelDragRow(self) -> str:
        return "<tr class='top-level-drag-row'><td colspan='6'>&nbsp;</td></tr>"

    # Options
    ##########################################################################

    def _showOptions(self, did: str) -> None:
        m = QMenu(self.mw)
        a = m.addAction(tr.actions_rename())
        assert a is not None
        qconnect(a.triggered, lambda b, did=did: self._rename(DeckId(int(did))))
        a = m.addAction(tr.actions_options())
        assert a is not None
        qconnect(a.triggered, lambda b, did=did: self._options(DeckId(int(did))))
        a = m.addAction(tr.actions_export())
        assert a is not None
        qconnect(a.triggered, lambda b, did=did: self._export(DeckId(int(did))))
        a = m.addAction(tr.actions_delete())
        assert a is not None
        qconnect(a.triggered, lambda b, did=did: self._delete(DeckId(int(did))))
        gui_hooks.deck_browser_will_show_options_menu(m, int(did))
        m.popup(QCursor.pos())

    def _export(self, did: DeckId) -> None:
        self.mw.onExport(did=did)

    def _rename(self, did: DeckId) -> None:
        def prompt(name: str) -> None:
            new_name = getOnlyText(
                tr.decks_new_deck_name(), default=name, title=tr.actions_rename()
            )
            if not new_name or new_name == name:
                return
            else:
                rename_deck(
                    parent=self.mw, deck_id=did, new_name=new_name
                ).run_in_background()

        QueryOp(
            parent=self.mw, op=lambda col: col.decks.name(did), success=prompt
        ).run_in_background()

    def _options(self, did: DeckId) -> None:
        display_options_for_deck_id(did)

    def _collapse(self, did: DeckId) -> None:
        node = self.mw.col.decks.find_deck_in_tree(self._render_data.tree, did)
        if node:
            node.collapsed = not node.collapsed
            set_deck_collapsed(
                parent=self.mw,
                deck_id=did,
                collapsed=node.collapsed,
                scope=DeckCollapseScope.REVIEWER,
            ).run_in_background()
            self._renderPage(reuse=True)

    def _handle_drag_and_drop(self, source: DeckId, target: DeckId) -> None:
        reparent_decks(
            parent=self.mw, deck_ids=[source], new_parent=target
        ).run_in_background()

    def _delete(self, did: DeckId) -> None:
        deck = self.mw.col.decks.find_deck_in_tree(self._render_data.tree, did)
        assert deck is not None
        deck_name = deck.name
        remove_decks(
            parent=self.mw, deck_ids=[did], deck_name=deck_name
        ).run_in_background()

    # Top buttons
    ######################################################################

    drawLinks = [
        ["", "practice_exam", "Do Practice Exam"],
        ["", "shared", tr.decks_get_shared()],
        ["", "create", tr.decks_create_deck()],
        ["Ctrl+Shift+I", "import", tr.decks_import_file()],
    ]

    def _drawButtons(self) -> None:
        buf = ""
        drawLinks = deepcopy(self.drawLinks)
        for b in drawLinks:
            if b[0]:
                b[0] = tr.actions_shortcut_key(val=shortcut(b[0]))
            buf += """
<button title='%s' onclick='pycmd(\"%s\");'>%s</button>""" % tuple(b)
        self.bottom.draw(
            buf=buf,
            link_handler=self._linkHandler,
            web_context=DeckBrowserBottomBar(self),
        )

    def _onShared(self) -> None:
        openLink(f"{aqt.appShared}decks/")

    def _on_create(self) -> None:
        if op := add_deck_dialog(
            parent=self.mw, default_text=self.mw.col.decks.current()["name"]
        ):
            op.run_in_background()

    def _on_practice_exam(self) -> None:
        from aqt.practiceexam import display_practice_exam

        display_practice_exam(self.mw)

    def _on_set_exam_date(self) -> None:
        """Prompt for the target exam date (YYYY-MM-DD) and store it in config.
        The memory and performance metrics are projected to this date."""
        import time
        from datetime import datetime

        current = self.mw.col.get_config("mcatExamDate", None)
        default = ""
        if current:
            default = time.strftime("%Y-%m-%d", time.localtime(int(current)))
        text = getOnlyText(
            "Enter your exam date (YYYY-MM-DD):",
            parent=self.mw,
            default=default,
        )
        if not text.strip():
            return
        try:
            parsed = datetime.strptime(text.strip(), "%Y-%m-%d")
        except ValueError:
            showInfo("Please enter the date as YYYY-MM-DD.", parent=self.mw)
            return
        if parsed.date() < datetime.now().date():
            showInfo(
                "Exam date must be today or in the future.",
                parent=self.mw,
            )
            return
        self.mw.col.set_config("mcatExamDate", int(parsed.timestamp()))
        self.refresh()

    ######################################################################

    def _v1_upgrade_message(self, required: bool) -> str:
        if not required:
            return ""

        update_required = tr.scheduling_update_required().replace("V2", "v3")

        return f"""
<center>
<div class=callout>
    <div>
      {update_required}
    </div>
    <div>
      <button onclick='pycmd("v2upgrade")'>
        {tr.scheduling_update_button()}
      </button>
      <button onclick='pycmd("v2upgradeinfo")'>
        {tr.scheduling_update_more_info_button()}
      </button>
    </div>
</div>
</center>
"""

    def _confirm_upgrade(self) -> None:
        if self.mw.col.sched_ver() == 1:
            self.mw.col.mod_schema(check=True)
            self.mw.col.upgrade_to_v2_scheduler()
        self.mw.col.set_v3_scheduler(True)

        showInfo(tr.scheduling_update_done())
        self.refresh()
