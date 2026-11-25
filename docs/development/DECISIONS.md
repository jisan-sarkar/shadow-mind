# Preparation decisions and approval gates

| Decision | Status and required evidence |
|---|---|
| Reference host and workspace | Ubuntu 26.04.1 LTS, GNOME/Wayland, x86-64. Existing private origin cloned to the agreed workspace. Host and branch evidence is in [STATUS.md](STATUS.md). |
| Host prerequisites | Owner approved the named APT and per-user toolchain changes in chat and applied APT from the regular Ubuntu terminal. All approved prerequisites are verified. A native probe that needs an additional system package requires a separate transaction review. |
| Exact dependency versions | FEAS-04 records the compatible pins, locked builds and native test results. The probe found no use of the affected GLib iterator. The owner approved the tested pins for D02, with the advisory visible in audits and reviewed against the D02 build and before release. Material baseline changes require an owner-approved decision. |
| D00 and D01 exits | Owner approved D00 exit in chat after PREP-01–04 evidence review. D01 prototypes are isolated. The owner approved D01 exit in chat after the native consent checks and dependency review. BUILD-PLAN and the D02 foundation are eligible. |
| Sources and permissions | Synthetic inputs by default. Approve actual test sources and any participant involvement before using them. Document permitted source compatibility in D01–D03. |
| Provider and spending | Mock only. Authorized paid budget is USD 0. Resolve provider/model choice in D04. Real content transmission and nonzero spend each require explicit authorization and the applicable policy controls. |
| Repository protection and remote actions | Manual reviews and passing checks pending owner approval of configuration changes. Obtain the named approval before pushes, merges or publication. |
| Benchmarks and release | Measure in D01–D07 and obtain owner ratification of thresholds. Validate packaging in D08 and obtain D09 private release approval. Public brand, namespace and license decisions precede public distribution. |

The project owner remains accountable for R01–R12. The implementing agent gathers the engineering, AI, security and performance evidence at the resolution gates in specification §19. The earliest risks to resolve are R03 (hardware) and R04 (Ubuntu compatibility) through D00/D01. R05 (data exposure) and R07 (charges) require decisions before external or paid processing. Review the remaining risks at the specified milestone exits.
