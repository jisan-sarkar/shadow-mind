# ShadowMind — project setup

Status: approved direction; execution pending on the owner's Ubuntu computer. This file supplements PROJECT_SPECIFICATION.md v1.4, especially sections 22–28. It does not authorize host changes or production implementation.

## 1. Identity

Product: ShadowMind; repository/package/executable slug: shadow-mind; application ID: com.shadowmind.desktop (private-development selection; verify namespace before public release). Initial software version 0.1.0. Private repository: https://github.com/jisan-sarkar/shadow-mind. Main is stable; dev is integration. No public license selected.

## 2. Inspect the actual host (PREP-01)

Run on the owner's Ubuntu system, not an unrelated cloud runner. Report only necessary sanitized outputs.

```bash
cat /etc/os-release
uname -m
printf '%s\n' "$XDG_CURRENT_DESKTOP" "$XDG_SESSION_TYPE"
lscpu
free -h
lspci | grep -Ei 'vga|3d|display'
df -h /
git --version
node --version
pnpm --version
rustc --version
cargo --version
pkg-config --modversion webkit2gtk-4.1
```

Record missing commands; do not install them during inspection. Target the existing supported Ubuntu LTS; do not upgrade the OS merely for the project.

## 3. Repository and toolchain (PREP-02/03)

If a local clone does not already exist, authenticate using the owner's approved GitHub method and clone the private repository:

```bash
git clone https://github.com/jisan-sarkar/shadow-mind.git
cd shadow-mind
git fetch origin
git switch dev
git status --short --branch
```

For an existing clone, inspect remote, branches and uncommitted changes; do not overwrite the user's work. Never put tokens in command arguments or committed files. Confirm the repository documentation PR state before branching.

Review current official prerequisites at https://v2.tauri.app/start/prerequisites/ and the generator at https://v2.tauri.app/start/create-project/. Candidate Ubuntu packages include build-essential, pkg-config, libwebkit2gtk-4.1-dev, libssl-dev, librsvg2-dev, libxdo-dev and libayatana-appindicator3-dev. Check actual availability and simulate installation before asking for approval. Example dry run:

```bash
apt-get -s install build-essential pkg-config libwebkit2gtk-4.1-dev libssl-dev librsvg2-dev libxdo-dev libayatana-appindicator3-dev
```

After explicit host-change approval, use the supported distro package manager and reviewed official Rust/Node/pnpm installation methods. Prefer stable Rust, compatible Node LTS (Node 24 candidate) and a pinned pnpm release. Record exact versions after D01. Do not use unreviewed install scripts, blanket upgrades or elevated privileges for normal application operation.

## 4. Feasibility then scaffolding

Complete D00 exit review first. During D01, use an isolated disposable scaffold generated with the official Tauri v2 React/TypeScript template; do not overwrite this repository. Validate ordinary desktop build, standard permission behavior and mock-only processing with synthetic fixtures. Once D01 passes, integrate the verified scaffold on a feature branch in D02, preserving root documents. Use the official generator or supported existing-project initialization rather than a hand-assembled Tauri setup.

Identity values: productName ShadowMind; identifier com.shadowmind.desktop; npm and Rust package shadow-mind; software version 0.1.0. Pin validated versions in package.json, rust-toolchain.toml and lockfiles; commit pnpm-lock.yaml and Cargo.lock. Dependencies beyond the scaffold require a justified task. Candidate quality tools: ESLint, Prettier, Vitest, React Testing Library, rustfmt, Clippy and cargo test. Add Tailwind, Zustand and necessary UI components incrementally.

No real input collection or paid AI requests in normal tests; no capture circumvention, hidden overlays or covert assessment functionality.

## 5. Quality gates and commands (after scaffold)

Configure the following scripts/checks and run them from the appropriate directory; they are not available yet:

- pnpm install --frozen-lockfile (after lockfile creation)
- pnpm lint; pnpm typecheck; pnpm test; pnpm build (define actual scripts first)
- cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
- cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings
- cargo test --manifest-path src-tauri/Cargo.toml --locked
- pnpm tauri build (after native prerequisites and package validation)

Record actual outputs and failures, not just commands. Pin reviewed GitHub Actions to full commit SHAs, least-privilege contents: read for normal CI, no paid API credentials in tests, no auto-publishing. Native Ubuntu checks require the actual reference host.

## 6. GitHub settings — owner action

In repository settings, keep Private, enable Issues, enable squash merges, disable merge commits and rebase merges if desired, and enable deletion of merged feature branches only when it will not delete the persistent dev branch. Keep auto-merge off. Configure main/dev protection or rulesets where supported, require pull requests and passing checks once CI exists, prohibit force pushes and deletion of protected branches. Do not require an independent review from a solo maintainer without an available reviewer. If private-repository rulesets are unavailable on the current plan, follow the same checks manually. Use GitHub Issues and milestones D00–D09. Never claim settings are applied until inspected.

## 7. D00 exit

- Actual host inventory recorded.
- Missing prerequisites and authorized changes documented.
- Repository identity and access confirmed; local clone verified.
- Master specification and agent documentation present.
- Mock-only, zero paid spend confirmed; no credentials exposed.
- D01 feasibility tests and evidence locations identified.
- Remaining blockers and owner decisions recorded.

D00 exit requires owner review; no production code before D00 and D01 approval. See PROJECT_STATUS.md for current state.
