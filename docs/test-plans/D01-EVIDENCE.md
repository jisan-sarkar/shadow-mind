# D01 feasibility evidence

Reference environment checked 2026-11-25 (Asia/Dhaka). Ubuntu 26.04.1 LTS, GNOME/Wayland, x86-64, Core i5-8365U, UHD 620, 14 GiB RAM. WebKitGTK 2.52.6, GTK 3.24.52, ScreenCast portal interface version 5, PipeWire 1.6.2, GNOME Keyring 50. All specimens were synthetic. Paid use was USD 0.

Specification baseline `c33a54af33b297b07bf15dab32611675e814e72a`. The official Tauri generator created `shadow-mind-feasibility` in isolated local scratch. The probe identity is `com.shadowmind.feasibility`. The production repository contains these evidence records and the D00 handover. The isolated source, exact lockfiles and reproduction helpers are under `/home/js/Documents/Codex/2026-11-25/new-chat/work/shadowmind-d01.0JhwRNIl`.

## Results

| Check | Status | Evidence and limit |
|---|---|---|
| FZ01 locked toolchain | Pass | Fresh frontend workspace installed with `pnpm install --frozen-lockfile`. ESLint, `tsc --noEmit`, Vite build, `cargo fmt --check`, locked Clippy and locked Cargo tests passed. The first locked Tauri build produced a Debian debug bundle. Both lockfiles matched their recorded hashes after bundling. |
| FZ02 shell | Pass | The non-root probe ran with `GDK_BACKEND=wayland`, emitted a ready marker from React over Tauri IPC and closed with exit 0. The native window reported successful resize, minimize, and restore requests. The initial two-run cleanup probe found no remaining processes in the probe session after close. |
| FZ03 UI | Pass for fixture checks | In the native WebKitGTK app, DOM checks confirmed the rendered synthetic text, code and equation, focus on both button targets, different computed light/dark styles, and increased text scale. Focus changes were exercised programmatically. Physical Tab navigation remains a separate manual observation. |
| FZ04 screen authorization | Pass for one permitted window | The owner dismissed the first GNOME prompt and the portal returned no stream. In the allow test, the owner selected the synthetic window. The probe matched its in-memory fixture color at the same sample point in successive frames and closed the portal session. In the live revocation test, GNOME emitted the session Closed signal after the owner stopped sharing. The screen pipeline was released and the probe exited 0. No grant was persisted. |
| FZ05 audio authorization | Pass for synthetic source | A private PipeWire graph and policy-only WirePlumber instance used a generated tone. Denial left no consumer. The approved synthetic node had the active link. Source removal ended the link, and teardown removed the consumer. No physical microphone was used. |
| FZ06 Both | Pass for one permitted screen and isolated synthetic audio | With the native portal screen stream active, the denied audio path created no consumer. The granted fixture linked only to its selected audio node. Removing that node ended the audio link and removed the consumer. The visible probe status reported the loss, and a subsequent frame still matched the permitted screen fixture. |
| FZ07 protected/unavailable input | Pass for synthetic safe failure | GStreamer-generated red and black frames were classified as available and unavailable respectively. The unavailable path released resources and opened no alternate source. |
| FZ08 credentials | Pass | libsecret stored, looked up and removed a uniquely tagged dummy entry in GNOME Secret Service. A fresh process with an inaccessible session bus reported service unavailable without a fallback. Dummy values stayed out of output. |
| FZ09 resources | Measured | Twenty native optimized-build runs passed. See the measurements and limitations below. |
| FZ10 mock provider | Pass | Native React/Tauri IPC and two Rust tests exercised deterministic success, malformed result, timeout and cancellation after start. A `strace -f -e trace=connect` rerun observed zero IPv4/IPv6 connection attempts. |
| FZ11 Debian package | Build, install simulation and staged smoke passed | Both builds produced amd64 Debian artifacts. Package and desktop metadata checks passed. The optimized artifact resolved in a read-only APT simulation and launched from a clean `dpkg-deb --extract` staging directory. System installation, upgrade and removal remain separate acceptance procedures. |
| FZ12 coexistence | Pass for mock meeting | The staged packaged probe launched and closed while a separate GTK mock-meeting window remained mapped and intact under GNOME/Wayland. Primary conferencing-product validation is scheduled for D07. |

## Resource baseline and package identity

Used the optimized build from `pnpm exec tauri build --bundles deb -- --locked` with opt-level 2, LTO disabled, 16 codegen units and two build jobs. Both lockfiles matched their pre-build checksums. Repeated launches used fresh processes and the same disposable profile after the first launch. The host page cache was left as-is. Startup ended at a ready event from the rendered React tree through native IPC. The fixture batch included the deliberate mock timeout. Teardown started after the window operation report and ended with the app and its helper processes gone.

| Measurement | Samples | Median | p95 |
|---|---:|---:|---:|
| Process start to WebView ready | 20 | 674.9 ms | 957.0 ms |
| WebView ready to fixture report | 20 | 92.4 ms | 130.6 ms |
| Window operation report to probe process group exit | 20 | 76.6 ms | 130.6 ms |

The first launch with a new profile took 591.9 ms. There were zero recorded failures in the 20 runs. One 5.02-second idle sample recorded 172.7 MiB proportional set size across the app and WebKit helpers, with no observed CPU ticks in that interval at the available measurement granularity. These measurements are for an empty synthetic probe. User-interaction latency, production pause/stop acknowledgement, active media/model loads and the four-hour soak remain separate measurements.

Optimized package `shadow-mind-feasibility` 0.1.0, amd64, 1,982,136 bytes. SHA-256 `d51d3cf405fffbcd4ae1bf77ab87a8521770cae1ef145e39d7a9ee085179ce53`. `apt-get -s --no-remove install` proposed only the local package, with zero upgrades and removals. The staged smoke run exercised the WebKit fixtures and recorded zero IPv4/IPv6 connect attempts.


## Version candidates and audit

| Component | Resolved probe version |
|---|---|
| rustup, Rust and Cargo | 1.29.3, 1.100.0 and 1.100.0 |
| Node and pnpm | 24.22.3 and 12.18.0 |
| Official generator | 4.7.5 |
| Tauri crate, build crate, CLI and JS API | 2.12.7, 2.7.6, 2.12.6 and 2.12.5 |
| React/React DOM, Vite, React plugin and TypeScript | 19.3.1, 8.3.8, 6.3.0 and 6.0.3 |
| Tokio, ESLint and TypeScript ESLint parser | 1.54.1, 10.15.0 and 8.74.0 |

The published Tauri crates declare a Rust MSRV of 1.90 and Tokio 1.85. The tested toolchain is Rust 1.100.0 and the highest declared Linux dependency MSRV is 1.90. All 285 resolved Linux dependency manifests had license metadata. The JavaScript license inventory was generated for later distribution review. pnpm audit reported no known advisories. RustSec 0.23.0 loaded the advisory database and completed the lockfile scan. Its crates.io index refresh reported an error. The scan reported these three warnings:

- [RUSTSEC-2026-0543](https://rustsec.org/advisories/RUSTSEC-2026-0543) for unmaintained `chrono` 0.4.45. The Linux target dependency tree did not contain `chrono`.
- [RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370) for unmaintained build-time `proc-macro-error` 1.0.4 via `glib-macros`.
- [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429) for unsound `glib::VariantStrIter` in `glib` 0.18.5 through the Tauri Linux stack. The source review found the API declaration and GLib's own tests, with no calls in the probe or retrieved dependents. The unstripped debug binary had no matching iterator symbols. This evidence applies to the feasibility build. The reviewed upstream Tauri manifest continues to require GTK Rust bindings 0.18, so the patched GLib 0.20 line is outside that dependency range. See the [upstream Tauri issue](https://github.com/tauri-apps/tauri/issues/12048) and [Tauri manifest](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/Cargo.toml).

## Interactive test limitations

Two allow attempts returned an authorized stream and closed it safely after a false-negative fixture check. The output had black margins, and only one sampled point matched the exact fixture color. An animated marker and successive-frame agreement at the matching point made the test reliable. The first manual revocation attempt timed out and the app closed the share. The repeat observed GNOME's native revocation signal and exited cleanly. The screen probe retained no frames.

## FEAS-04 proposed disposition

The isolated build and the authorized procedures provide D01 feasibility evidence. Approve the tested Tauri v2 pins for the local D02 foundation. Keep the GLib advisory and the two maintenance warnings visible in dependency scans. Before release, re-evaluate the linked Linux dependency graph and affected paths on the actual product build, check for an upstream fix, and document remediation or an approved disposition under the specification's security and release gates. This proposal does not approve production release or suppress the advisory.

The owner approved D01 exit and the tested pins for D02 in chat. The GLib advisory remains tracked for the D02 build and release review.
