import contextlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi, GLib

if len(sys.argv) != 2:
    raise SystemExit("usage: python3 scripts/native-smoke.py path/to/shadow-mind")

BINARY = str(Path(sys.argv[1]).resolve())


def walk(node):
    queue = [node]
    count = 0
    while queue and count < 200:
        item = queue.pop(0)
        if not item:
            continue
        count += 1
        yield item
        try:
            queue.extend(item.get_child_at_index(i) for i in range(max(0, item.get_child_count())))
        except GLib.Error:
            continue


def root_for(pid):
    desktop = Atspi.get_desktop(0)
    for index in range(desktop.get_child_count()):
        child = desktop.get_child_at_index(index)
        try:
            if child and child.get_process_id() == pid:
                return child
        except GLib.Error:
            continue
    return None


def named(root, name, role=None):
    if root is None:
        return None
    for current in walk(root):
        try:
            if role is not None and current.get_role_name() != role:
                continue
            if current.get_name() == name:
                return current
            if role is None and "Text" in current.get_interfaces():
                iface = current.get_text_iface()
                size = iface.get_character_count()
                if 0 < size < 300 and Atspi.Text.get_text(iface, 0, size) == name:
                    return current
        except GLib.Error:
            continue
    return None


def wait_until(callback, description):
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        value = callback()
        if value:
            return value
        time.sleep(0.05)
    raise AssertionError("Missing " + description)


def click(accessible):
    action = accessible.get_action_iface()
    assert action and action.get_n_actions() > 0, accessible.get_name()
    assert action.do_action(0), accessible.get_name()


@contextlib.contextmanager
def launch(profile, log):
    settings = profile / "config"
    with log.open("wb") as output:
        process = subprocess.Popen(
            [BINARY],
            env={
                **os.environ,
                "GDK_BACKEND": "wayland",
                "XDG_CONFIG_HOME": str(settings),
                "XDG_DATA_HOME": str(profile / "data"),
                "XDG_CACHE_HOME": str(profile / "cache"),
            },
            stdout=output,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        try:
            app = wait_until(lambda: root_for(process.pid), "app accessibility tree")
            yield process, app
        except Exception:
            if root_for(process.pid):
                for node in walk(root_for(process.pid)):
                    try:
                        print(node.get_role_name(), node.get_name(), flush=True)
                    except GLib.Error:
                        continue
            print(log.read_text(errors="replace")[-3500:], flush=True)
            raise
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=8)


def smoke():
    Atspi.init()
    with tempfile.TemporaryDirectory(prefix="shadowmind-d02-smoke-") as tmp:
        profile = Path(tmp)
        log = profile / "native.log"
        settings = profile / "config" / "com.shadowmind.desktop" / "settings.json"
        with launch(profile, log) as (process, app):
            def require(name, role=None):
                return wait_until(lambda: named(app, name, role), name)

            continue_button = require("Continue to workspace", "button")
            component = continue_button.get_component_iface()
            assert component.grab_focus()
            wait_until(lambda: require("Continue to workspace", "button").get_state_set()
                       .contains(Atspi.StateType.FOCUSED), "native focus")
            assert not settings.exists()
            click(continue_button)
            for choice in ("Audio Spoken context", "Both Visual and spoken context", "Screen Visual context"):
                option = require(choice, "radio button")
                click(option)
                wait_until(lambda: require(choice, "radio button").get_state_set()
                           .contains(Atspi.StateType.CHECKED), "mode selection")
                click(require("Configure session", "button"))
                require("Start demo", "button")
                assert require(choice, "radio button").get_state_set().contains(Atspi.StateType.CHECKED)
                click(require("Cancel setup", "button"))
                require("Configure session", "button")
            click(require("Configure session", "button"))
            start = require("Start demo", "button")
            assert not start.get_state_set().contains(Atspi.StateType.SENSITIVE)
            click(require("Use bundled synthetic inputs for this session.", "check box"))
            wait_until(lambda: require("Start demo", "button").get_state_set()
                       .contains(Atspi.StateType.SENSITIVE), "confirmation enabling start")
            click(require("Start demo", "button"))
            require("Demo active")
            click(require("Run fixture", "button"))
            require("How can I turn meeting notes into clear action items?")
            click(require("Pause", "button"))
            require("Paused")
            click(require("Resume", "button"))
            require("Demo active")
            click(require("Stop session", "button"))
            require("Stopped")
            assert named(app, "How can I turn meeting notes into clear action items?") is None
            saved = json.loads(settings.read_text())
            assert saved == {
                "schemaVersion": 1,
                "onboardingComplete": True,
                "appearance": {"theme": "system", "textScale": 100},
            }
            click(require("Configure session", "button"))
            click(require("Use bundled synthetic inputs for this session.", "check box"))
            click(require("Start demo", "button"))
            require("Demo active")
            click(require("Close", "button"))
            assert process.wait(timeout=8) == 0
        with launch(profile, log) as (_, app):
            wait_until(lambda: named(app, "Configure session", "button"), "persisted onboarding")
            wait_until(lambda: named(app, "Idle"), "idle after relaunch")
            assert named(app, "Demo active") is None
        settings.write_text("{invalid")
        with launch(profile, log) as (_, app):
            wait_until(lambda: named(app, "Saved settings could not be loaded. ShadowMind opened with defaults."), "recovery notice")
            wait_until(lambda: named(app, "Continue to workspace", "button"), "recovered onboarding")
            assert settings.read_text() == "{invalid"
        print("PASS native focus, onboarding, three modes, confirmation, mock response, pause/resume/stop, window close, clean restart and corrupt settings recovery")


if __name__ == "__main__":
    smoke()
