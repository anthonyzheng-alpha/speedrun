# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

import aqt
import aqt.main
from aqt.qt import *
from aqt.utils import disable_help_button, restoreGeom, saveGeom
from aqt.webview import AnkiWebView, AnkiWebViewKind


class PracticeExamDialog(QDialog):
    "Configure and take a randomly-generated MCAT practice exam."

    TITLE = "practiceExam"
    silentlyClose = True

    def __init__(self, mw: aqt.main.AnkiQt) -> None:
        QDialog.__init__(self, mw, Qt.WindowType.Window)
        self.mw = mw
        self._setup_ui()

    def _setup_ui(self) -> None:
        self.setWindowModality(Qt.WindowModality.ApplicationModal)
        self.mw.garbage_collect_on_dialog_finish(self)
        self.setMinimumWidth(400)
        self.setMinimumHeight(500)
        disable_help_button(self)
        restoreGeom(self, self.TITLE, default_size=(800, 800))

        self.web = AnkiWebView(kind=AnkiWebViewKind.PRACTICE_EXAM)
        self.web.set_bridge_command(self._link_handler, self)
        self.web.load_sveltekit_page("practice-exam")
        layout = QVBoxLayout()
        layout.setContentsMargins(0, 0, 0, 0)
        layout.addWidget(self.web)
        self.setLayout(layout)
        self.setWindowTitle("Practice Exam")
        self.show()

    def _link_handler(self, url: str) -> None:
        if url == "refresh_home_metrics" and self.mw.state == "deckBrowser":
            self.mw.deckBrowser.refresh()

    def _refresh_home_if_visible(self) -> None:
        if self.mw.state == "deckBrowser":
            self.mw.deckBrowser.refresh()

    def reject(self) -> None:
        self.web.cleanup()
        self.web = None  # type: ignore
        saveGeom(self, self.TITLE)
        self._refresh_home_if_visible()
        QDialog.reject(self)


def display_practice_exam(mw: aqt.main.AnkiQt) -> None:
    PracticeExamDialog(mw)
