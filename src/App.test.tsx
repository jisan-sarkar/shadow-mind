import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import App from "./App";
import {
  chooseImage,
  dispatchSession,
  getSnapshot,
  savePreferences,
  subscribeSnapshots,
  type Snapshot,
} from "./bridge";

vi.mock("./bridge", async (load) => ({
  ...(await load<typeof import("./bridge")>()),
  getSnapshot: vi.fn(),
  chooseImage: vi.fn(),
  savePreferences: vi.fn(),
  dispatchSession: vi.fn(),
  subscribeSnapshots: vi.fn(),
}));

const idle: Snapshot = {
  schemaVersion: 1,
  revision: 0,
  preferences: {
    schemaVersion: 1,
    onboardingComplete: true,
    appearance: { theme: "system", textScale: 100 },
  },
  recoveryNotice: null,
  session: {
    id: null,
    mode: null,
    phase: "idle",
    fixtureStatus: "ready",
    screenStatus: "idle",
    audioStatus: "idle",
    fixture: null,
  },
};

let publish: (snapshot: Snapshot) => void;

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(subscribeSnapshots).mockImplementation(async (handler) => {
    publish = handler;
    return () => undefined;
  });
  vi.mocked(getSnapshot).mockResolvedValue(idle);
});

afterEach(() => {
  cleanup();
  delete document.documentElement.dataset.theme;
  delete document.documentElement.dataset.scale;
});

test("onboarding persists only after the user continues", async () => {
  const firstLaunch: Snapshot = {
    ...idle,
    recoveryNotice: "invalidSettings",
    preferences: { ...idle.preferences, onboardingComplete: false },
  };
  vi.mocked(getSnapshot).mockResolvedValue(firstLaunch);
  vi.mocked(savePreferences).mockResolvedValue({ ...idle, revision: 1 });
  render(<App />);
  const continueButton = await screen.findByRole("button", {
    name: "Continue to workspace",
  });
  expect(screen.getByText(/saved settings could not be loaded/i)).toBeTruthy();
  expect(dispatchSession).not.toHaveBeenCalled();
  fireEvent.click(continueButton);
  expect(savePreferences).toHaveBeenCalledWith(
    idle.preferences.appearance,
    true,
  );
  expect(
    await screen.findByRole("button", { name: "Configure session" }),
  ).toBeTruthy();
  expect(screen.queryByText(/saved settings could not be loaded/i)).toBeNull();
});

test("mode selection requires confirmation for its own session", async () => {
  const configured: Snapshot = {
    ...idle,
    revision: 1,
    session: {
      ...idle.session,
      id: "session-test",
      mode: "both",
      phase: "configuring",
    },
  };
  const active: Snapshot = {
    ...configured,
    revision: 2,
    session: { ...configured.session, phase: "active" },
  };
  vi.mocked(dispatchSession)
    .mockResolvedValueOnce(configured)
    .mockResolvedValueOnce(active);
  render(<App />);
  await screen.findByRole("button", { name: "Configure session" });
  fireEvent.click(screen.getByRole("radio", { name: /^Both/ }));
  fireEvent.click(screen.getByRole("button", { name: "Configure session" }));
  expect(dispatchSession).toHaveBeenCalledWith({
    type: "configure",
    mode: "both",
  });

  const start = (await screen.findByRole("button", {
    name: "Start demo",
  })) as HTMLButtonElement;
  expect(start.disabled).toBe(true);
  fireEvent.click(
    screen.getByRole("checkbox", { name: /Use bundled synthetic inputs/ }),
  );
  expect(start.disabled).toBe(false);
  fireEvent.click(start);
  expect(dispatchSession).toHaveBeenLastCalledWith({
    type: "start",
    sessionId: "session-test",
    confirmFixtures: true,
  });
  expect(await screen.findByText("Demo active")).toBeTruthy();
});

test("a stale fixture event cannot return a response to a stopped session", async () => {
  const active: Snapshot = {
    ...idle,
    revision: 1,
    session: {
      ...idle.session,
      id: "session-test",
      mode: "audio",
      phase: "active",
    },
  };
  vi.mocked(getSnapshot).mockResolvedValue(active);
  vi.mocked(dispatchSession).mockResolvedValue({
    ...active,
    revision: 3,
    session: { ...active.session, phase: "stopped" },
  });
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Stop session" }));
  expect(await screen.findByText("Stopped")).toBeTruthy();
  act(() =>
    publish({
      ...active,
      revision: 2,
      session: {
        ...active.session,
        fixtureStatus: "complete",
        fixture: { question: "Fixture question", answer: "Stale result" },
      },
    }),
  );
  expect(screen.queryByText("Stale result")).toBeNull();
  expect(screen.getByText("Stopped")).toBeTruthy();
});

test("settings errors surface and a later save updates appearance", async () => {
  const appearance = { theme: "light", textScale: 125 } as const;
  vi.mocked(savePreferences)
    .mockRejectedValueOnce({
      code: "settings_write_failed",
      message: "Settings could not be saved.",
    })
    .mockResolvedValueOnce({
      ...idle,
      revision: 1,
      preferences: { ...idle.preferences, appearance },
    });
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Settings" }));
  fireEvent.change(screen.getByLabelText("Theme"), {
    target: { value: "light" },
  });
  fireEvent.change(screen.getByLabelText("Text size"), {
    target: { value: "125" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save settings" }));
  expect((await screen.findByRole("alert")).textContent).toBe(
    "Settings could not be saved.",
  );
  expect(document.documentElement.dataset.theme).toBe("system");
  fireEvent.click(screen.getByRole("button", { name: "Save settings" }));
  await waitFor(() =>
    expect(document.documentElement.dataset.theme).toBe("light"),
  );
  expect(document.documentElement.dataset.scale).toBe("125");
});

test("image inspection requires separate authorization for each selection", async () => {
  vi.mocked(getSnapshot).mockResolvedValue({
    ...idle,
    revision: 1,
    session: {
      ...idle.session,
      id: "allowed-image-session",
      mode: "screen",
      phase: "active",
    },
  });
  vi.mocked(chooseImage).mockResolvedValue({
    format: "PNG",
    width: 32,
    height: 24,
  });
  render(<App />);
  const choose = (await screen.findByRole("button", {
    name: "Choose image",
  })) as HTMLButtonElement;
  expect(choose.disabled).toBe(true);
  fireEvent.click(
    screen.getByRole("checkbox", {
      name: /I have permission to inspect the image/,
    }),
  );
  fireEvent.click(choose);
  expect(chooseImage).toHaveBeenCalledWith("allowed-image-session", true);
  expect(await screen.findByText("Valid PNG · 32 × 24 pixels")).toBeTruthy();
  expect(choose.disabled).toBe(true);
});
