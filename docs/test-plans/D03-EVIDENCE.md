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

D03 is locally implemented with partial native verification. Preserve the remaining acceptance items and evidence before declaring exit. Use a fabricated window and audio fixture, or explicitly authorized sources. D03 sends no input to an AI provider.
