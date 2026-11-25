# ADR-011 · D02 native foundation and dependency disposition

## Status

Accepted for local D02 work by the owner after D01 review on 2026-11-25.

## Context

Specification ADR-001–ADR-010 establish the desktop architecture and policy. D01 verified the locked Tauri/WebKitGTK stack on the reference Ubuntu host, including its ordinary permission workflow. The Linux Rust graph includes GLib 0.18.5. The D01 source and linked debug binary checks found no use of the affected iterator API. [D01 evidence](../test-plans/D01-EVIDENCE.md) · [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2024-0429.html)

## Decision

Use the tested Tauri v2 pins for the D02 foundation. Preserve advisory visibility in dependency scans and record the actual D02 build assessment. Evaluate a supported upstream update when available and review affected paths before release. A backport or toolkit migration requires approval and a fresh native compatibility run.

Rust owns the session and versioned preferences. A small atomic JSON file meets the current persistence need. The frontend consumes typed snapshots and a targeted update event. Synthetic fixtures reside in the application bundle and exercise the session path without adding a network client.

## Consequences

D02 can verify an ordinary app shell, settings recovery and the session workflow on the approved platform. D03 supplies native capture adapters and OS lifecycle behavior. Release approval still depends on the canonical security gates and the advisory disposition.
