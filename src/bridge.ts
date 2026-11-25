import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Theme = "system" | "light" | "dark";
export type TextScale = 100 | 112 | 125;
export type InputMode = "screen" | "audio" | "both";
export type Phase =
  "idle" | "configuring" | "active" | "paused" | "stopping" | "stopped";
export type FixtureStatus =
  "ready" | "processing" | "complete" | "cancelled" | "failed";

export interface Appearance {
  theme: Theme;
  textScale: TextScale;
}

export interface SessionSnapshot {
  id: string | null;
  mode: InputMode | null;
  phase: Phase;
  fixtureStatus: FixtureStatus;
  fixture: { question: string; answer: string } | null;
}

export interface Snapshot {
  schemaVersion: 1;
  revision: number;
  preferences: {
    schemaVersion: 1;
    onboardingComplete: boolean;
    appearance: Appearance;
  };
  recoveryNotice: "invalidSettings" | "unreadableSettings" | null;
  session: SessionSnapshot;
}

type SessionCommand =
  "pause" | "resume" | "stop" | "runFixture" | "cancelFixture";

export type SessionAction =
  | { type: "configure"; mode: InputMode }
  | { type: "start"; sessionId: string; confirmFixtures: boolean }
  | { type: SessionCommand; sessionId: string };

/** Request the Rust-owned state for the current local window. */
export const getSnapshot = () => invoke<Snapshot>("snapshot");

/** Save validated appearance and, on explicit confirmation, finish onboarding. */
export const savePreferences = (
  appearance: Appearance,
  completeOnboarding: boolean,
) =>
  invoke<Snapshot>("update_preferences", {
    update: { appearance, completeOnboarding },
  });

/** Apply a guarded operation to the current synthetic session. */
export const dispatchSession = (action: SessionAction) =>
  invoke<Snapshot>("session_action", { action });

/** Receive versioned updates for local asynchronous fixture completion. */
export function subscribeSnapshots(update: (snapshot: Snapshot) => void) {
  return listen<{
    schemaVersion: number;
    kind: string;
    origin: string;
    snapshot: Snapshot;
  }>("shadowmind:snapshot", ({ payload }) => {
    if (
      payload.schemaVersion === 1 &&
      payload.kind === "snapshotUpdated" &&
      payload.origin === "session"
    ) {
      update(payload.snapshot);
    }
  });
}

export function errorMessage(error: unknown): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    "code" in error &&
    typeof error.message === "string" &&
    typeof error.code === "string"
  ) {
    return error.message;
  }
  return "ShadowMind could not complete this action. Try again.";
}
