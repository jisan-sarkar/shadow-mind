# Development workflow

ShadowMind is a private, specification-driven solo project with agent-assisted development.

1. Read PROJECT_SPECIFICATION.md and AGENTS.md; inspect PROJECT_STATUS.md and Git status.
2. Confirm the next task's prerequisites and acceptance criteria.
3. Branch from dev using feature/, fix/ or docs/ prefixes; keep changes focused.
4. Use Conventional Commits and reference requirement or task IDs.
5. Run applicable checks and attach sanitized evidence to the PR; disclose unrun tests and blockers.
6. Squash reviewed feature changes into dev; promote verified release-ready changes to main via a reviewed PR. Preserve main and dev; no force pushes.
7. Update documentation, traceability and status as needed.

Use strict TypeScript, ESLint/Prettier, rustfmt/Clippy, pinned dependencies and committed lockfiles after scaffold validation. Use synthetic fixtures and deterministic mocks for routine CI. No secrets, raw recordings or confidential inputs in source, logs, issues or PRs. No new scope or paid services without approval. The master specification defines readiness, definition of done, defect severity and release gates.
