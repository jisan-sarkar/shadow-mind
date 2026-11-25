# D03 input verification

Reference environment is Ubuntu 26.04.1 on GNOME/Wayland, WebKitGTK 2.52.6, GStreamer 1.28.2 and PipeWire 1.6.2. Development used a scratch SDK extracted from the matching Ubuntu development archives. Build linkage and packaged execution were verified against the already installed runtime. The local repository is on `feature/d03-authorized-inputs`, based on merged D02 commit `1c5d840`.

| Area | Tested outcome |
|---|---|
| Frontend | ESLint, Prettier, TypeScript and all frontend tests passed. The image control requires separate confirmation before every selection. |
| Static image validation | Rust decoded valid PNG/JPEG fixtures and rejected malformed, empty, unsupported and oversized inputs, and URI forms for nonlocal sources. Session tests verify that disallowed modes, stale IDs and duplicate requests fail, and pause invalidates a pending request. The normal desktop picker opened. Selection and display of a file through that picker remain pending interactive verification. |
| Audio | The packaged application enumerated a private PipeWire graph containing a generated tone, required selection and permission, reported receipt, then showed Unavailable when the synthetic source disconnected. The graph had no remaining active link. A focused GStreamer test also verified bounded input and pipeline state returning to Null. |
| Screen | The packaged application initiated a normal GNOME sharing request with no previous grant. Stopping during Connecting returned the slot to Off and exited cleanly. Positive selection, receipt and user-triggered revocation still need the owner-controlled synthetic window test on this implementation. |
| Both and cleanup | Rust tests verify independent source state, continued screen status after audio loss, suppression of stale callbacks and the no-restart-until-cleanup guard. An app-level combined native test and physical lock, sleep and minimize tests remain pending. |
| Packaging | The locked amd64 debug Debian bundle built, extracted and launched. Runtime libraries resolved from the system. The native D02 workspace regression smoke passed on the D03 build. |
| Dependencies | RustSec reported the existing GTK stack advisory and two maintenance warnings. No new advisories were reported by that scan. Production release gates and provider validation remain pending. |

## D03 cleanup revision

The 2026-11-26 check used the reference GNOME/Wayland desktop, Node 24.22.3 and the matching scratch GStreamer development SDK. The debug amd64 package SHA-256 was `8c43516d2523fa928ae9c97a3d97bc97ae84316660baca4c2d450b6d00df6838`. It was extracted into a temporary directory and launched with a disposable profile.

| Check | Actual outcome |
|---|---|
| Frontend lint, formatting, type-check and tests | Passed, 5 tests. |
| Rust formatting, locked Clippy and tests | Passed, 14 tests. Regression coverage includes late image completion, release-failure restart blocking, lock-state changes and both portal revocation event orders. |
| Locked debug Debian bundle | Built and extracted successfully. |
| D02 packaged desktop regression | Passed focus, onboarding, three modes, explicit confirmation, synthetic result, pause/resume/stop, window close, clean restart and settings recovery. |
| D03 pending screen request | Passed. App stop withdrew the portal request and returned status to Off. |
| D03 audio-only | Passed. The private PipeWire graph delivered the authorized generated tone, reported source loss and had no remaining active link. |
| D03 combined | Screen receipt continued after audio loss. An earlier GNOME revocation check timed out. The dedicated revocation rerun passed, as detailed below. |
| D03 native PNG and JPEG selection | The PNG result reached the expected validated dimensions. The first JPEG attempt timed out. The dedicated JPEG and cancellation rerun passed, as detailed below. |
| D03 minimize | Before the desktop notification change, the GNOME/Wayland minimize test left the synthetic audio link active. The new notification Stop input action requires verification in the minimized state. |
| Frontend dependency audit | No known vulnerabilities reported by `pnpm audit`. |
| RustSec audit | Exit 0 on 479 crates. Three inherited warnings are `RUSTSEC-2024-0429`, `RUSTSEC-2026-0543` and `RUSTSEC-2024-0370`. The advisory database loaded. The crates.io index refresh reported a local configuration warning. |

The notification revision produced debug bundle SHA-256 `f12323d956d655d2460ecc260d4fe739799ba5e702f7db2d0534def6ed65eb6a`. All 19 automated tests, lint, type checking and locked Clippy passed. The extracted package passed the D02 regression, pending portal cancel, isolated audio disconnection, audio pause, and active-audio window close. In each audio teardown check the private PipeWire graph showed no remaining active link.

The first notification revision turned off screen input when the audio connection closed. The follow-up separated notification lifetime from capture connections. The dedicated `both-isolation` case passed against debug bundle SHA-256 `f655314f0234e060f8a3addb713250566c248145032e19e04e238d4c3520900b`. Both sources reached Receiving locally. After the tone disconnected, audio reported Unavailable, the private PipeWire graph had no active link, and the screen still reported Receiving locally. ShadowMind's Stop live inputs control returned both sources to Off and the app closed normally.

The `revoke-first` case passed on the same bundle under GNOME/Wayland. With both synthetic sources receiving, GNOME's Stop Sharing action ended screen capture and the app reported Permission ended. Audio still reported Receiving locally and its PipeWire link remained active. Disconnecting the tone then set audio to Unavailable and released its link. The runner exited successfully.

The `denial` case passed on the same bundle. Canceling the initial request from GNOME's sharing dialog set the screen status to Permission ended and removed the live-input stop control. After fresh consent, a second request reached Connecting. Stopping that request in ShadowMind returned the screen to Off and the runner exited successfully.

The `close` case passed on the same bundle. The runner selected the synthetic window through the normal GNOME dialog with user consent, verified Receiving locally, and observed a new PipeWire video node. It closed ShadowMind through the window control and verified exit code 0 and removal of the video node while the synthetic window was still running. The GNOME top-bar indicator is a separate visual check.

The `minimize` case remains open on the same bundle. The first run timed out with the synthetic audio link still active. In two retries the link disappeared, but the runner could not verify Off in the restored workspace. The monitored attempt did not show the Stop input action signal during the test. The runner now checks that audio is active after minimizing and rejects process exit as evidence that the notification stopped capture. A separate pause regression passed.

GNOME suppresses banners for resident notifications while their application has focus. The follow-up removes the resident hint and keeps a zero expiry. Rust formatting, Clippy, all 14 Rust tests, and the debug build passed. The minimized desktop rerun on SHA-256 `0b49393f58a3445ead13e9104f60d2210ae0d65d6274fa34a14914b267390f6f` timed out with audio still active. The D-Bus monitor observed no Stop input action during that run. A confirmed notification click and restoration are still required. [GNOME Shell implementation](https://raw.githubusercontent.com/GNOME/gnome-shell/50.1/js/ui/notificationDaemon.js)

On a later run with the same bundle, the monitor saw GNOME receive the notification with the Stop input button. The desktop runner observed the audio link disappear, reported Off in the workspace, and exited successfully. The monitor did not capture the button's action signal. A follow-up with an explicit focus check after restore timed out while audio was still active. The isolated D-Bus test passed and verified that the application accepts Stop input for its own notification ID and ignores actions for another ID or a different button.

The `image-jpeg-cancel` case passed on GNOME/Wayland with bundle SHA-256 `0b49393f58a3445ead13e9104f60d2210ae0d65d6274fa34a14914b267390f6f`. Canceling the first native chooser removed the waiting status and cleared consent. After fresh consent, the same session opened a new picker and displayed Valid JPEG with the expected 32 × 24 dimensions. The runner exited successfully. Four focused Rust image tests and all five frontend tests passed.

The original lock check reached a real GNOME lock while the synthetic audio link remained active. The updated build subscribes to LockedHint changes on the resolved logind session. The complete `lock` case passed with bundle SHA-256 `4f63ff4314092bb46842dad4ab87eb0199bbcf6a6b794d6f4f9768fc7ff1db73`. The audio link disappeared and the image picker's dedicated D-Bus connection closed while the desktop was locked and ShadowMind was still running. After normal unlock, ShadowMind stayed Paused. Explicit Resume left audio Off and required fresh consent for audio and image input. Rust formatting, locked Clippy, and all 14 regular Rust tests passed.

The reference portal advertised window and display selection, and the PipeWire inventory reported two Audio/Source nodes. Display capture and a source scoped to an ordinary application still need acceptance evidence.

## Repeatable desktop acceptance

Build the debug bundle with the locked dependencies, extract it without installing it, and pass the extracted binary to the smoke scripts from the repository root.

```sh
pnpm exec tauri build --debug --bundles deb -- --locked
stage=$(mktemp -d)
dpkg-deb -x src-tauri/target/debug/bundle/deb/ShadowMind_0.1.0_amd64.deb "$stage"
python3 scripts/native-smoke.py "$stage/usr/bin/shadow-mind"
SHADOWMIND_INPUT_TEST=cancel python3 scripts/native-input-smoke.py "$stage/usr/bin/shadow-mind"
SHADOWMIND_INPUT_TEST=audio python3 scripts/native-input-smoke.py "$stage/usr/bin/shadow-mind"
dbus-run-session -- cargo test --manifest-path src-tauri/Cargo.toml --locked only_the_matching_stop_action_ends_capture -- --ignored
```

The tests use GNOME/Wayland and Python GObject bindings for AT-SPI, GTK and GdkPixbuf. The audio cases also use PipeWire, WirePlumber and the GStreamer PipeWire plug-in. The runner creates the synthetic window, tone, PNG and JPEG and uses disposable profiles. Run each remaining case by substituting its value for `SHADOWMIND_INPUT_TEST`.

| Case | Desktop action and expected result |
|---|---|
| `denial` | Cancel the GNOME share dialog. The app reports Permission ended and clears the pending request. The runner verifies a fresh request can start, then cancels it through ShadowMind. |
| `screen` | Select only ShadowMind D03 Synthetic Window. The app receives samples and app-driven stop returns the source to Off. |
| `close` | Select the synthetic window. The runner closes the app during capture and verifies successful exit and removal of the PipeWire video node while the fixture stays open. Confirm GNOME's sharing indicator clears. |
| `both-isolation` | Select the synthetic window. The runner disconnects the tone and verifies that audio is Unavailable, the audio link is gone, and screen stays active. It then stops both sources through ShadowMind. |
| `both` | Select the synthetic window. The runner disconnects the tone and checks that screen stays active. Stop sharing via GNOME and check that the app reports Permission ended for the screen. |
| `revoke-first` | Select the synthetic window, then stop sharing via GNOME when prompted. The runner verifies audio stays active before disconnecting it. |
| `image`, `image-png`, `image-jpeg` | Choose both generated images in turn or a single format. Check the metadata result. |
| `image-jpeg-cancel` | Cancel the first picker. The runner checks that the request and consent clear. Select the generated JPEG in the reopened picker and verify its metadata. |
| `pause`, `close-audio` | The runner pauses or closes the app with the synthetic audio link active and checks that the link is released. |
| `minimize` | Open GNOME notifications from the top-bar clock and choose Stop input on ShadowMind audio input. Restore the app when prompted. The runner checks graph cleanup and Off status. |
| `lock` | Lock GNOME with the picker open. Wait about 10 seconds and unlock normally. The runner checks that the audio link and picker connection close while locked, then verifies explicit resume and fresh consent. |
| `sleep` | Return after sleeping as prompted. The runner checks audio teardown and verifies that explicit resume leaves the source Off. |

On the reference desktop, repeat lock/unlock and suspend/resume with the image chooser open. Check that pending requests are withdrawn, late results are discarded, and reopening the chooser requires a fresh confirmation. Verify screen cleanup through the GNOME sharing indicator and audio cleanup through the private PipeWire graph. Test a full display only on a controlled desktop showing synthetic content. Record the supported microphone and authorized application-audio choices, including any capabilities the host does not expose.

D03 remains open pending native acceptance. Input stays local to the app.
