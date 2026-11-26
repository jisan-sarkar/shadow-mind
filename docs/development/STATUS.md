# Development handover and D00 evidence

| Field | Record |
|---|---|
| Date and environment | 2026-11-25 (Asia/Dhaka), reference Ubuntu desktop, active GNOME/Wayland session |
| Specification and task | v1.4. D00 and D01 exits approved. BUILD-PLAN and D02 local foundation verified. The [D02 report](../test-plans/D02-EVIDENCE.md) records verification. |
| Workspace | `/home/js/Projects/shadow-mind`, local branch `feature/d02-foundation` from `dev` |
| Baseline | `origin/dev` at `c33a54af33b297b07bf15dab32611675e814e72a` and `origin/main` at `fd98f54c60af97a182e4792e4ba2f36b555e5643` |
| Relevant requirements | NFR09/NFR11 and SEC07/SEC08 prerequisite evidence. No requirement test executed. |
| Blocker and next task | Review the D02 branch and integrate through an authorized GitHub PR. D03 is the next implementation milestone. Keep the GLib advisory visible for release review. |
| Changes in this handover | Locked Tauri foundation, settings, synthetic lifecycle, frontend and native checks, CI workflow, traceability and evidence. |

## PREP-01 reference host

The independent terminal inventory corroborates the owner-provided record. The host runs Ubuntu 26.04.1 LTS (Resolute), x86-64. `loginctl` confirmed an active Wayland session, and GNOME Shell was present on the user bus. The Intel Core i5-8365U has four cores and eight threads, with Intel UHD Graphics 620. Linux reported 14 GiB usable RAM, 5.7 GiB available at this inspection, 9.5 GiB swap and 75 GiB free on the root filesystem.

| Inspection | Outcome |
|---|---|
| `git --version`, `gh --version` | 2.53.0, 2.105.0 |
| `node --version`, `npm --version`, `pnpm --version` | 24.22.0, 12.2.0, 12.18.0 |
| `gcc --version`, `pkg-config --version`, `pkg-config --modversion gtk+-3.0` | 15.2.0, 2.5.1, 3.24.52 |
| `command -v rustup rustc cargo` | All absent from the inspected shell |
| Individual `pkg-config --modversion` checks | WebKitGTK 4.1 and OpenSSL development metadata unavailable |
| `dpkg-query -W` | WebKitGTK 4.1 runtime `2.52.6-0ubuntu0.26.04.2`, PipeWire `1.6.2-1ubuntu1.2`, WirePlumber `0.5.13-1ubuntu1.1`, GNOME Keyring `50.0-1`, system portal `1.21.1+ds-1ubuntu3.1`, GNOME portal backend `50.0-0ubuntu1` |
| User service and D-Bus checks | Both portals, PipeWire, WirePlumber and GNOME Keyring active. ScreenCast interface version 5. Native permission behavior remains a D01 test. |

## PREP-02 prerequisite assessment

APT candidate metadata and the read-only solver run succeeded. Keep the existing compiler, `build-essential`, `pkg-config`, curl, wget and file. Candidate sources below are the configured Ubuntu Resolute APT archives.

| Package absent at inspection | Candidate | APT source |
|---|---|---|
| `libwebkit2gtk-4.1-dev` | `2.52.6-0ubuntu0.26.04.2` | resolute-updates/main |
| `libssl-dev` | `3.5.5-1ubuntu3.7` | resolute-updates/main and resolute-security/main |
| `librsvg2-dev` | `2.61.3+dfsg-3ubuntu0.1` | resolute-updates/main and resolute-security/main |
| `libxdo-dev` | `1:3.20160805.1-5.1build1` | resolute/universe |
| `libayatana-appindicator3-dev` | `0.5.94-1build1` | resolute/main |

```sh
apt-get -s --no-remove install libwebkit2gtk-4.1-dev libssl-dev   librsvg2-dev libxdo-dev libayatana-appindicator3-dev
```

The simulated transaction returned exit 0 with 25 new packages, one update and zero removals. The dependency update was `libsqlite3-0`, from `3.46.1-9ubuntu0.3` to `3.46.1-9ubuntu0.4`. The independent `libpipewire-0.3-dev` simulation during planning proposed 12 audio-stack upgrades and two additions. Reassess it separately if the D01 native probe needs headers.

The owner approved the named host changes in chat. The reviewed official rustup installer and existing nvm completed the authorized per-user installations.

| Check | Outcome |
|---|---|
| `rustup --version`, `rustup run 1.100.0 rustc --version` | rustup 1.29.3 and rustc 1.100.0 on x86_64-unknown-linux-gnu |
| `cargo +1.100.0 --version`, `cargo +1.100.0 fmt --version`, `cargo +1.100.0 clippy --version` | Cargo 1.100.0, rustfmt 1.11.0-stable and Clippy 0.1.100 |
| Node, npm and pnpm version checks | Node 24.22.3, its bundled npm 11.19.1, and the reused pnpm 12.18.0 with the patched Node first on PATH |
| `sudo -n true` | Exit 1. The agent process reports `NoNewPrivs: 1` and cannot elevate. The owner completed the APT transaction from their regular Ubuntu terminal. Installation was independently checked afterward. |

The new Node version is side by side with the existing runtime. Source the rustup environment for Rust tools. For the development shell, select Node 24.22.3 through nvm and keep its bin directory before the existing pnpm launcher's directory on PATH. Post-install `dpkg-query -W` showed all five packages fully installed at the candidate versions above, and `libsqlite3-0` at `3.46.1-9ubuntu0.4`. The repeat APT simulation proposed zero upgrades, additions or removals for the requested package set. `dpkg --audit` returned no findings.

`pkg-config --modversion` resolved WebKitGTK 4.1 (2.52.6), OpenSSL (3.5.5), GTK 3 (3.24.52), librsvg (2.61.3) and Ayatana AppIndicator (0.5.94). A disposable C program compiled with `gcc -Wall -Wextra -Werror`, linked using `pkg-config --cflags --libs webkit2gtk-4.1 openssl` and `-lxdo`, and printed versions for WebKitGTK, OpenSSL and libxdo. This verifies toolchain linking for those libraries. D01 application build, native permissions and runtime checks remain Not Run.

## PREP-03 workspace and repository

Authenticated GitHub CLI inspection and SSH read, clone and fetch succeeded. The initial local branch was clean, tracked `origin/dev`, and matched its commit. Verified the eight baseline documentation files, Git configuration files, issue templates and PR template. The package slug is `shadow-mind`, the private-development application ID is `com.shadowmind.desktop`, and the initial version is 0.1.0.

The remote is private, `main` is the default, and Issues and Actions are enabled. Squash merges are enabled. Merge commits, rebase merges, auto-merge and automatic source-branch deletion are disabled. The branch APIs reported neither branch protected and no applicable rulesets. Use the reviewed-PR workflow and manual quality gate pending an owner decision. Foundation PRs #1 and #11 are merged. D00/D01 issues #2–#10 are open. Changes in this handover are local.

The newer status evidence takes precedence for task progress where the master specification still lists the host as unverified. Its v1.4 requirements and milestone gates remain authoritative. The documented HTTPS clone is an example. The verified SSH workflow was used for this checkout.

## PREP-04 and PREP-05 handoff

The development budget is USD 0 for paid AI. The [D01 procedures](../test-plans/D01.md) cover FZ01–FZ12 and specify synthetic fixtures and permission boundaries. [DECISIONS.md](DECISIONS.md) assigns the remaining risks and approvals to their resolution gates. The [traceability register](../requirements/TRACEABILITY.md) preserves all baseline requirement/control IDs and reserved test IDs.

The owner approved D00 exit in chat. Outstanding D01 decisions include native permissions and source compatibility, exact dependency pins and the feasibility resource baseline. If the screen/audio probe needs PipeWire development packages, evaluate their separate transaction before installation.
