"""Exercise the packaged desktop app with synthetic inputs and normal GNOME consent."""

from contextlib import contextmanager
import json
import os
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import time

import gi

gi.require_version("GdkPixbuf", "2.0")
from gi.repository import GdkPixbuf

SMOKE = runpy.run_path(str(Path(__file__).with_name("native-smoke.py")))
Atspi = SMOKE["Atspi"]
TONE = "ShadowMind_D03_Synthetic_Audio"
WINDOW = "ShadowMind D03 Synthetic Window"
LIVE_STATUSES = (
    "Off", "Connecting", "Releasing source", "Receiving locally", "Unavailable",
    "Permission ended", "Suspended", "Release failed. Reopen ShadowMind.",
    "Desktop stop controls unavailable",
)
WINDOW_FIXTURE = """
import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk
window = Gtk.Window(title="ShadowMind D03 Synthetic Window")
window.set_default_size(480, 320)
area = Gtk.DrawingArea()
colors = iter(([0.45, 0.2, 0.8], [0.7, 0.3, 0.3]) * 10000)
color = next(colors)
def tick():
    global color
    color = next(colors)
    area.queue_draw()
    return True
def draw(_widget, context):
    context.set_source_rgb(*color)
    context.paint()
    return False
area.connect("draw", draw)
window.add(area)
window.show_all()
GLib.timeout_add(200, tick)
Gtk.main()
"""


def wait_for(predicate, description, seconds=8):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if value := predicate():
            return value
        time.sleep(0.1)
    raise AssertionError(f"Timed out waiting for {description}")


@contextmanager
def child(arguments, env, log):
    with log.open("wb") as output:
        process = subprocess.Popen(arguments, env=env, stdout=output, stderr=subprocess.STDOUT)
        try:
            yield process
        except Exception:
            print(log.name, log.read_text(errors="replace")[-1500:], flush=True)
            raise
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)


@contextmanager
def virtual_audio(root):
    runtime = root / "pipewire"
    runtime.mkdir(mode=0o700)
    env = {
        **os.environ,
        "PIPEWIRE_RUNTIME_DIR": str(runtime),
        "XDG_RUNTIME_DIR": str(runtime),
        "XDG_STATE_HOME": str(root / "state"),
        "XDG_CONFIG_HOME": str(root / "pipewire-config"),
    }

    def graph():
        return json.loads(subprocess.check_output(["pw-dump"], env=env, timeout=4))

    with child(["pipewire"], env, root / "pipewire.log"):
        wait_for(lambda: (runtime / "pipewire-0").is_socket(), "private PipeWire daemon")
        with child(["dbus-run-session", "--", "wireplumber", "--profile=policy"],
                   env, root / "policy.log"):
            command = [
                "gst-launch-1.0", "-q", "audiotestsrc", "is-live=true", "freq=440",
                "!", "audio/x-raw,format=S16LE,rate=48000,channels=1",
                "!", "pipewiresink", "mode=provide",
                f"stream-properties=props,node.name=shadowmind.d03.tone,node.description={TONE},media.class=Audio/Source",
            ]
            with child(command, env, root / "tone.log") as tone:
                wait_for(lambda: any(TONE in str(node) for node in graph()), "synthetic tone")
                previous = os.environ.get("PIPEWIRE_RUNTIME_DIR")
                os.environ["PIPEWIRE_RUNTIME_DIR"] = str(runtime)
                try:
                    yield tone, lambda: any(
                        node.get("type") == "PipeWire:Interface:Link"
                        and node.get("info", {}).get("state") == "active"
                        for node in graph()
                    )
                finally:
                    if previous is None:
                        os.environ.pop("PIPEWIRE_RUNTIME_DIR", None)
                    else:
                        os.environ["PIPEWIRE_RUNTIME_DIR"] = previous


class Workspace:
    def __init__(self, app, mode):
        self.app = app
        self.click("Continue to workspace")
        if mode is not None:
            self.click(mode, "radio button")
        self.click("Configure session")
        self.click("Use bundled synthetic inputs for this session.", "check box")
        self.click("Start demo")

    def find(self, name, role=None):
        return SMOKE["named"](self.app, name, role)

    def expect(self, name, seconds=8):
        return wait_for(lambda: self.find(name), name, seconds)

    def expect_status(self, source, name, seconds=8):
        def status():
            section = self.find(f"{source} input", "landmark")
            return section and SMOKE["named"](section, name)

        try:
            return wait_for(status, f"{source} status {name}", seconds)
        except AssertionError as failure:
            section = self.find(f"{source} input", "landmark")
            if section:
                for actual in LIVE_STATUSES:
                    if SMOKE["named"](section, actual):
                        raise AssertionError(f"{source} status was {actual!r}, expected {name!r}") from failure
            raise

    def click(self, name, role="button"):
        def ready():
            item = self.find(name, role)
            return item if item and item.get_state_set().contains(Atspi.StateType.SENSITIVE) else None

        SMOKE["click"](wait_for(ready, name))

    def start_window(self):
        self.click("I have permission to share the window or display I select.", "check box")
        self.click("Choose window or display")
        self.expect_status("Screen", "Connecting")

    def start_audio(self):
        self.click("Find audio sources")
        self.click(TONE, "radio button")
        self.click("I have permission to use this audio source, including participant consent where required.",
                   "check box")
        self.click("Start audio input")

    def close(self, process):
        self.click("Close")
        assert process.wait(timeout=10) == 0


def screen_case(root, case):
    with child([sys.executable, "-c", WINDOW_FIXTURE], os.environ.copy(), root / "window.log"):
        with SMOKE["launch"](root, root / "app.log") as (process, app):
            workspace = Workspace(app, None)
            workspace.start_window()
            if case == "denial":
                print("Cancel the GNOME share dialog.", flush=True)
                workspace.expect("Permission ended", 75)
            elif case != "cancel":
                print(f"Select only {WINDOW} in the GNOME share dialog.", flush=True)
                workspace.expect("Receiving locally", 75)
            if case == "close":
                workspace.close(process)
            else:
                if case != "denial":
                    workspace.click("Stop live inputs")
                    workspace.expect("Off")
                    assert not workspace.find("Receiving locally")
                workspace.close(process)
            print(f"PASS screen {case}", flush=True)


def audio_case(root, case):
    combined = case != "audio"
    with virtual_audio(root) as (tone, has_active_link):
        with SMOKE["launch"](root, root / "app.log") as (process, app):
            mode = "Both Visual and spoken context" if combined else "Audio Spoken context"
            workspace = Workspace(app, mode)
            workspace.start_audio()
            workspace.expect_status("Audio", "Receiving locally")
            wait_for(has_active_link, "synthetic PipeWire link")
            if combined:
                workspace.start_window()
                print(f"Select only {WINDOW} in the GNOME share dialog.", flush=True)
                workspace.expect_status("Screen", "Receiving locally", 75)
                if case == "revoke-first":
                    print("Stop sharing using the GNOME sharing indicator.", flush=True)
                    workspace.expect_status("Screen", "Permission ended", 75)
                    workspace.expect_status("Audio", "Receiving locally")
                    assert has_active_link()
            tone.terminate()
            tone.wait(timeout=5)
            workspace.expect_status("Audio", "Unavailable")
            wait_for(lambda: not has_active_link(), "release of synthetic PipeWire link")
            if case in {"both", "both-isolation"}:
                workspace.expect_status("Screen", "Receiving locally")
                print("PASS the screen stayed active after audio disconnected.", flush=True)
            if case == "both-isolation":
                workspace.click("Stop live inputs")
                workspace.expect_status("Screen", "Off")
                workspace.expect_status("Audio", "Off")
            if case == "both":
                print("Stop sharing using the GNOME sharing indicator.", flush=True)
                workspace.expect_status("Screen", "Permission ended", 75)
            workspace.close(process)
            print("PASS isolated audio disconnection and resource release", flush=True)
            if case in {"both", "revoke-first"}:
                print("PASS independent combined inputs and GNOME revocation", flush=True)


def audio_lifecycle_case(root, action):
    with virtual_audio(root) as (_, has_active_link):
        with SMOKE["launch"](root, root / "app.log") as (process, app):
            workspace = Workspace(app, "Audio Spoken context")
            workspace.start_audio()
            workspace.expect_status("Audio", "Receiving locally")
            wait_for(has_active_link, "synthetic PipeWire link")
            if action == "close-audio":
                workspace.close(process)
            elif action in {"lock", "sleep"}:
                print(f"Use GNOME to {action} the desktop, then return to it.", flush=True)
            else:
                workspace.click("Pause" if action == "pause" else "Minimize")
                if action == "minimize":
                    print("Open GNOME notifications and choose Stop input on ShadowMind audio input.", flush=True)
            wait_for(lambda: not has_active_link(), f"PipeWire link release on {action}", 60)
            if action != "close-audio":
                if action == "minimize":
                    print("Restore the ShadowMind window to check that audio is Off.", flush=True)
                if action != "minimize" or workspace.find("Paused"):
                    workspace.expect("Paused", 60)
                    workspace.click("Resume")
                workspace.expect_status("Audio", "Off")
                assert not has_active_link()
                workspace.close(process)
            print(f"PASS {action} releases audio and requires fresh authorization", flush=True)


def image_case(root, formats):
    fixture = GdkPixbuf.Pixbuf.new(GdkPixbuf.Colorspace.RGB, False, 8, 32, 24)
    fixture.fill(0x6837c7ff)
    with SMOKE["launch"](root, root / "app.log") as (process, app):
        workspace = Workspace(app, None)
        for codec in formats:
            suffix = "jpg" if codec == "jpeg" else codec
            path = root / f"synthetic-image.{suffix}"
            fixture.savev(str(path), codec, [], [])
            workspace.click("I have permission to inspect the image I select.", "check box")
            workspace.click("Choose image")
            print(f"Choose this file in the GNOME file chooser: {path}", flush=True)
            workspace.expect(f"Valid {codec.upper()} · 32 × 24 pixels", 75)
            print(f"PASS {codec.upper()} selected in the native file chooser", flush=True)
        workspace.close(process)


def main():
    Atspi.init()
    case = os.environ.get("SHADOWMIND_INPUT_TEST", "cancel")
    with tempfile.TemporaryDirectory(prefix="shadowmind-native-input-") as directory:
        root = Path(directory)
        if case in {"cancel", "denial", "screen", "close"}:
            screen_case(root, case)
        elif case == "audio":
            audio_case(root, case)
        elif case in {"both", "both-isolation", "revoke-first"}:
            with child([sys.executable, "-c", WINDOW_FIXTURE], os.environ.copy(), root / "window.log"):
                audio_case(root, case)
        elif case in {"image", "image-png", "image-jpeg"}:
            image_case(root, ("png", "jpeg") if case == "image" else (case.removeprefix("image-"),))
        elif case in {"pause", "close-audio", "minimize", "lock", "sleep"}:
            audio_lifecycle_case(root, case)
        else:
            raise SystemExit(f"Unknown SHADOWMIND_INPUT_TEST: {case}")


if __name__ == "__main__":
    main()
