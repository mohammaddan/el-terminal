#!/usr/bin/env python3
"""Nautilus extension: Open in El-Terminal.

Ubuntu 24.04 / Nautilus 46 + python3-nautilus 4.x MenuProvider API.
Launches el-terminal with --working-directory as a direct argv element
(never via a shell), so spaces, Unicode, and special characters are safe.
"""

from __future__ import annotations

import shutil
from typing import List

from gi.repository import Gio, GLib, GObject, Nautilus


class ElTerminalExtension(GObject.GObject, Nautilus.MenuProvider):
    def __init__(self) -> None:
        super().__init__()
        self._executable = shutil.which("el-terminal") or "el-terminal"

    def _folder_path(self, file_info: Nautilus.FileInfo) -> str | None:
        if file_info.get_uri_scheme() != "file":
            return None
        if not file_info.is_directory():
            return None
        location = file_info.get_location()
        if location is None:
            return None
        path = location.get_path()
        if not path:
            return None
        return path

    def _launch(self, path: str) -> None:
        # argv list — path is one argument; never interpolate into a shell.
        argv = [self._executable, "--working-directory", path]
        try:
            Gio.Subprocess.new(argv, Gio.SubprocessFlags.NONE)
        except GLib.Error as err:
            print(f"el-terminal-nautilus: failed to launch: {err}", flush=True)

    def _make_item(self, name: str, file_info: Nautilus.FileInfo) -> Nautilus.MenuItem:
        item = Nautilus.MenuItem(
            name=name,
            label="Open in El-Terminal",
            tip="Open a new El-Terminal window in this folder",
        )
        item.connect("activate", self._on_activate, file_info)
        return item

    def _on_activate(self, _menu: Nautilus.MenuItem, file_info: Nautilus.FileInfo) -> None:
        path = self._folder_path(file_info)
        if path is not None:
            self._launch(path)

    def get_file_items(
        self,
        files: List[Nautilus.FileInfo],
    ) -> List[Nautilus.MenuItem]:
        if len(files) != 1:
            return []
        file_info = files[0]
        if self._folder_path(file_info) is None:
            return []
        return [
            self._make_item(
                "ElTerminalNautilus::open_terminal_file",
                file_info,
            )
        ]

    def get_background_items(
        self,
        current_folder: Nautilus.FileInfo,
    ) -> List[Nautilus.MenuItem]:
        if self._folder_path(current_folder) is None:
            return []
        return [
            self._make_item(
                "ElTerminalNautilus::open_terminal_background",
                current_folder,
            )
        ]
