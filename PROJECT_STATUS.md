# ShadowMind — project status

Updated 2026-11-25. Baseline [PROJECT_SPECIFICATION.md](PROJECT_SPECIFICATION.md) v1.4.

| Milestone | Status |
|---|---|
| D00 · Preparation | Owner approved [PREP-05](https://github.com/jisan-sarkar/shadow-mind/issues/6). Reference Ubuntu host, local clone and toolchains verified. |
| D01 · Feasibility | Owner approved [FEAS-04](https://github.com/jisan-sarkar/shadow-mind/issues/10) and the tested pins for D02. [Feasibility evidence](docs/test-plans/D01-EVIDENCE.md) records native consent, synthetic fixtures and the first resource baseline. |
| D02 · Application foundation | Implemented and verified locally on the reference desktop. See the [foundation contract](docs/architecture/D02-FOUNDATION.md) and [verification evidence](docs/test-plans/D02-EVIDENCE.md). Merged into `dev` in [PR #12](https://github.com/jisan-sarkar/shadow-mind/pull/12). [Post-merge CI passed](https://github.com/jisan-sarkar/shadow-mind/actions/runs/45619391675). |
| D03 · Authorized native inputs | Implemented on the local feature branch. Test outcomes and pending native acceptance are tracked in the [D03 evidence](docs/test-plans/D03-EVIDENCE.md). |
| D04–D09 | Pending their milestone prerequisites. |

## Next action

Finish the remaining D03 native acceptance checks in an authorized desktop session. Use [the D03 contracts](docs/architecture/D03-INPUTS.md) and record the results before marking the milestone verified. Paid usage and external processing remain disabled.

The [decision register](docs/development/DECISIONS.md) tracks the GLib advisory and remaining approvals. The [traceability register](docs/requirements/TRACEABILITY.md) distinguishes the verified D02 behavior from later integration and release conditions.
