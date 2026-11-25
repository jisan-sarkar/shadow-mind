# ShadowMind — project status

Updated 2026-11-25. Baseline [PROJECT_SPECIFICATION.md](PROJECT_SPECIFICATION.md) v1.4.

| Milestone | Status |
|---|---|
| D00 · Preparation | Owner approved [PREP-05](https://github.com/jisan-sarkar/shadow-mind/issues/6). Reference Ubuntu host, local clone and toolchains verified. |
| D01 · Feasibility | Owner approved [FEAS-04](https://github.com/jisan-sarkar/shadow-mind/issues/10) and the tested pins for D02. [Feasibility evidence](docs/test-plans/D01-EVIDENCE.md) records native consent, synthetic fixtures and the first resource baseline. |
| D02 · Application foundation | Implemented and verified locally on the reference desktop. See the [foundation contract](docs/architecture/D02-FOUNDATION.md) and [verification evidence](docs/test-plans/D02-EVIDENCE.md). Hosted CI remains pending the reviewed remote integration. |
| D03 · Authorized native inputs | Next after foundation review. |
| D04–D09 | Pending their milestone prerequisites. |

## Next action

Review and integrate the D02 branch through the approved GitHub workflow. Start D03 with WP01/WP02/WP06, their typed source and grant contracts, and their authorized native acceptance cases. The D02 app uses bundled synthetic inputs and stores preferences locally. Paid usage and external processing remain disabled.

The [decision register](docs/development/DECISIONS.md) tracks the GLib advisory and remaining approvals. The [traceability register](docs/requirements/TRACEABILITY.md) distinguishes the verified D02 behavior from later integration and release conditions.
