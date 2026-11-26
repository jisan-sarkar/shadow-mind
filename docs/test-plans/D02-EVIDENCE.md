# D02 foundation verification

Verified on 2026-11-25 on the reference Ubuntu 26.04.1 GNOME/Wayland desktop, x86-64. WebKitGTK 2.52.6, Rust 1.100.0, Node 24.22.3 and pnpm 12.18.0. Source is on the local `feature/d02-foundation` branch from `dev` at baseline `c33a54af33b297b07bf15dab32611675e814e72a`.

| Area | Result |
|---|---|
| Locked frontend | Clean `pnpm install --frozen-lockfile`, ESLint, Prettier, TypeScript strict compilation, four Vitest/React Testing Library tests and Vite production build passed. Root build and clean checkout-equivalent build produced identical frontend output. |
| Locked Rust | rustfmt, Clippy with `-D warnings` and five Cargo tests passed. Tests cover consent, invalid or stale transitions, pending fixture cancellation, atomic settings persistence and malformed/oversize/schema recovery. |
| Native desktop | Started through `pnpm tauri dev --no-watch -- --locked`, verified native WebView onboarding and closed normally. Ran [native-smoke.py](../../scripts/native-smoke.py) against the staged packaged binary with a disposable local profile. |
| Native acceptance | The accessible control accepted focus. Fresh onboarding preceded configuration. All three modes could be selected before start, satisfying TC-FR01. Start required confirmation for the current session. The bundled response arrived through the asynchronous Rust event. Pause, resume and stop worked. Closing an active window exited cleanly. Relaunch was idle and retained only preferences. Corrupt settings recovered to defaults and surfaced a notice without overwriting the source file during load. |
| Egress and supply chain | Native synthetic workflow observed zero IPv4/IPv6 `connect` calls. pnpm audit reported no known advisories. RustSec completed the lockfile scan with the three previously recorded warnings and a crates.io index refresh warning. The unstripped D02 executable had no GLib iterator symbols, supporting the D02 assessment only. |
| Debian bundle | Locked build produced ShadowMind 0.1.0 for amd64. Both lockfile checksums stayed unchanged. Bundle SHA-256 is `93b9fb2438b4e4ab99ca74512d559750fcb248ceb795b8d9856c965500521d1c`. Package metadata and desktop file validation passed. The extracted binary passed native smoke testing. This is local development evidence. System installation, upgrade and rollback are D08 checks. |
| GitHub Actions | The pinned Ubuntu workflow is defined and its YAML was validated locally. Hosted execution is pending remote integration. |

The foundation checks exercise the D02 portions of FR12, FR23, FR37, FR43, FR44, FR55, FR60, FR62, FR78, FR79, ENG01–ENG07 and REL05. D03 still owns native source grants, minimized capture controls, lock/sleep handling and capture resource shutdown. D04 owns actual provider availability. Full release-gate and UI accessibility validation remains pending.

To repeat the native check, build and extract the Debian package into a temporary directory and run `python3 scripts/native-smoke.py /absolute/path/to/usr/bin/shadow-mind` on a GNOME/Wayland session with Python GI and AT-SPI installed. It launches the specified executable with disposable config, data and cache directories and inspects only that process's accessibility tree.
