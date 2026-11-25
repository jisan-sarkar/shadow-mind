# ShadowMind — project status

Updated 2026-11-25. Baseline [PROJECT_SPECIFICATION.md](PROJECT_SPECIFICATION.md) v1.4.

## D00 — Project preparation complete

| Task | Evidence and status |
|---|---|
| PREP-01 · [#2](https://github.com/jisan-sarkar/shadow-mind/issues/2) | Reference host inventory independently verified on the Ubuntu desktop. |
| PREP-02 · [#3](https://github.com/jisan-sarkar/shadow-mind/issues/3) | Verified. Owner installed the five native packages. Rust, Cargo, Node, pnpm, package metadata and native compiler/linker smoke checks passed. |
| PREP-03 · [#4](https://github.com/jisan-sarkar/shadow-mind/issues/4) | Private origin, clean initial `dev` checkout, identity, foundation files and settings verified. Local evidence and all 138 requirement mappings verified. |
| PREP-04 · [#5](https://github.com/jisan-sarkar/shadow-mind/issues/5) | Prepared. Mock-only policy, USD 0 paid budget, FZ01–FZ12 procedures, risk ownership and evidence locations recorded. |
| PREP-05 · [#6](https://github.com/jisan-sarkar/shadow-mind/issues/6) | Approved by the owner in chat following review of the verified prerequisite evidence. |

D01 feasibility evidence is ready for FEAS-04 owner review. D02–D09 remain pending. The application has no implementation, runtime evidence or release. Requirement tests and VG01–VG10 remain Not Run.

## Evidence and next action

- [Host, prerequisite and repository evidence](docs/development/STATUS.md)
- [Open decisions and resolution gates](docs/development/DECISIONS.md)
- [Requirement traceability](docs/requirements/TRACEABILITY.md)
- [D01 feasibility procedures](docs/test-plans/D01.md)
- [D01 feasibility evidence](docs/test-plans/D01-EVIDENCE.md)

Review and approve the D01 exit at FEAS-04. The remaining decision is whether to use the tested Tauri stack for D02 with the GNOME/GLib advisory tracked for affected-path reassessment and release disposition. Once approved, start BUILD-PLAN and D02.
