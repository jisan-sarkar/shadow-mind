# ShadowMind

Real-time contextual intelligence for authorized learning, collaboration and personal productivity.

ShadowMind is a planned Ubuntu-first desktop AI copilot. It will interpret permitted screen and audio inputs in an explicitly authorized session and provide contextual answers in a standard desktop workspace.

## Status

Specification baseline v1.4. The Ubuntu desktop workspace supports local settings and a synthetic demo session. D03 adds explicitly authorized native inputs and local image inspection. Provider integration follows in D04. No public software release is available.

## Stack

Tauri v2, Rust/Tokio, React, TypeScript, Vite and pnpm. Modular monolith; local-first data handling; optional approved AI provider. The toolchain files and lockfiles pin the tested foundation versions.checks.

## Run the foundation

Follow [PROJECT_SETUP.md](PROJECT_SETUP.md) for the Ubuntu prerequisites, including the D03 GStreamer build libraries. Use Node and Rust from the checked-in toolchain files and pnpm from `package.json`.

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

Complete first-launch setup, choose a mode, and confirm the bundled inputs before starting a demo. **Run fixture** displays the mock response. Preferences are saved locally. The session and its answer stay in memory.

| Command | Purpose |
|---|---|
| `pnpm lint`, `pnpm typecheck`, `pnpm test` | Frontend quality checks |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked` | Rust session and settings tests |
| `pnpm tauri build --bundles deb -- --locked` | Build the local Ubuntu package |

The [D02 foundation contract](docs/architecture/D02-FOUNDATION.md) documents how Rust owns session state and preferences.

## Start here

- [Project specification](PROJECT_SPECIFICATION.md): requirements, architecture, roadmap and acceptance gates.
- [Project setup](PROJECT_SETUP.md): host and repository preparation.
- [Agent instructions](AGENTS.md): coding agent entry point.
- [Project status](PROJECT_STATUS.md): current milestone and blockers.
- [Contributing](CONTRIBUTING.md) and [Security](SECURITY.md): development rules.

## Scope and privacy

Use only with permission to access and process the content, and with participant consent where required. No monitoring evasion, protected-content bypass, hidden capture-excluded overlays, or unauthorized assessment assistance. No raw input retention by default. Mock providers and synthetic fixtures are the development default; paid and external processing require approval.

Private development; no public license or release is granted. Public branding clearance remains pending.
