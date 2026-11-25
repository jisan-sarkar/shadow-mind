# BUILD-PLAN · D02 application foundation

D00 and D01 exits are owner-approved. Develop on the branch from `dev` using the tested Tauri stack. The production foundation uses only bundled synthetic inputs until the native adapters pass D03.

| Package | Contract and dependencies | D02 acceptance |
|---|---|---|
| E02 · Shell and workspace | One ordinary local Tauri WebView. Narrow IPC allowlist, versioned snapshots and an asynchronous update event. Depends on the locked D01 platform and dependency set. | Clean build and launch, fresh onboarding without capture, accessible configuration and active-session status. FR23, FR37, FR55, FR78, FR79, ENG01–ENG04. |
| E03 · Session lifecycle | Rust owns the state machine. An in-memory grant authorizes a single synthetic session. Pause and stop cancel work. Session and request identities gate late completion. | Verify configure/start/pause/resume/stop/close, explicit fixture confirmation, invalid and stale transitions, cancellation and clean restart. FR01, FR12, FR60, FR62, ENG06/ENG07. D03 adds native grants, minimization controls and OS interruption behavior. |
| E09 · Settings | Rust validates versioned, non-secret JSON preferences and writes them atomically. Appearance and onboarding are the persisted fields. No settings field grants capture, provider access or paid work. | Missing settings use safe defaults. Invalid or unsupported files surface a recovery notice. Failed saves preserve the previous state. Validate types and report mock availability. FR43, FR44, ENG05 and REL05. |

## IPC

The main local WebView can call `snapshot`, `update_preferences` and `session_action`. Each returns a snapshot with schema version 1 and an application revision. Commands carry a session ID after configuration. Public errors include a stable code, category, safe message and capability. No native handle, credential, path or captured content crosses IPC.

`shadowmind:snapshot` is a targeted event for a completed synthetic request. The UI applies snapshots in revision order. Synchronous state transitions and persistence run through a serialized Rust state. Mock execution is bounded to one request per session and is cancellable. A fresh app process starts with an idle session, even when it recovers saved preferences.

Launch tests exercise the real WebView. Unit and integration checks cover typed inputs, persistence, transition invariants and late cancellation. Requirements that depend on native capture, real provider setup or later permissions remain pending in traceability.
