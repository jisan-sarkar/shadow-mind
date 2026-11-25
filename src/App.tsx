import { useEffect, useState } from "react";
import {
  dispatchSession,
  errorMessage,
  getSnapshot,
  savePreferences,
  subscribeSnapshots,
  type Appearance,
  type SessionAction,
  type Snapshot,
  type TextScale,
  type Theme,
} from "./bridge";
import Workspace, { phaseLabels } from "./Workspace";
import "./app.css";

type View = "workspace" | "settings";

function Settings({
  appearance,
  busy,
  save,
}: {
  appearance: Appearance;
  busy: boolean;
  save: (appearance: Appearance) => Promise<void>;
}) {
  const [draft, setDraft] = useState(appearance);
  return (
    <div className="settings-grid">
      <section className="panel" aria-labelledby="appearance-heading">
        <h2 id="appearance-heading">Appearance</h2>
        <form
          className="settings-form"
          onSubmit={(event) => {
            event.preventDefault();
            void save(draft);
          }}
        >
          <label>
            Theme
            <select
              value={draft.theme}
              onChange={(event) =>
                setDraft({
                  ...draft,
                  theme: event.currentTarget.value as Theme,
                })
              }
            >
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
          </label>
          <label>
            Text size
            <select
              value={draft.textScale}
              onChange={(event) =>
                setDraft({
                  ...draft,
                  textScale: Number(event.currentTarget.value) as TextScale,
                })
              }
            >
              <option value={100}>100%</option>
              <option value={112}>112%</option>
              <option value={125}>125%</option>
            </select>
          </label>
          <button
            className="primary"
            disabled={
              busy ||
              (draft.theme === appearance.theme &&
                draft.textScale === appearance.textScale)
            }
          >
            Save settings
          </button>
        </form>
      </section>
      <section className="panel" aria-labelledby="privacy-heading">
        <h2 id="privacy-heading">Privacy and processing</h2>
        <dl className="session-details">
          <div>
            <dt>Screen and microphone access</dt>
            <dd>Off</dd>
          </div>
          <div>
            <dt>Remote processing</dt>
            <dd>Off</dd>
          </div>
          <div>
            <dt>Paid usage</dt>
            <dd>Off</dd>
          </div>
          <div>
            <dt>Session history</dt>
            <dd>In memory</dd>
          </div>
        </dl>
      </section>
    </div>
  );
}

export default function App() {
  const [current, setCurrent] = useState<Snapshot | null>(null);
  const [view, setView] = useState<View>("workspace");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [startupFailed, setStartupFailed] = useState(false);

  function apply(next: Snapshot) {
    if (next.schemaVersion !== 1)
      throw new Error("Unsupported workspace version");
    setCurrent((previous) =>
      !previous || next.revision > previous.revision ? next : previous,
    );
  }

  useEffect(() => {
    let live = true;
    let unsubscribe: (() => void) | undefined;
    const receive = (snapshot: Snapshot) => {
      if (live) apply(snapshot);
    };
    void (async () => {
      try {
        const dispose = await subscribeSnapshots(receive);
        if (!live) {
          dispose();
          return;
        }
        unsubscribe = dispose;
        receive(await getSnapshot());
      } catch {
        if (live) setStartupFailed(true);
      }
    })();
    return () => {
      live = false;
      unsubscribe?.();
    };
  }, []);

  const appearance = current?.preferences.appearance;

  useEffect(() => {
    if (!appearance) return;
    document.documentElement.dataset.theme = appearance.theme;
    document.documentElement.dataset.scale = String(appearance.textScale);
  }, [appearance]);

  async function run(operation: () => Promise<Snapshot>) {
    setBusy(true);
    setError("");
    try {
      apply(await operation());
    } catch (failure) {
      setError(errorMessage(failure));
    } finally {
      setBusy(false);
    }
  }

  if (!current) {
    return (
      <main className="initial-state">
        <h1>ShadowMind</h1>
        {startupFailed ? (
          <>
            <p role="alert">The workspace could not be opened.</p>
            <button
              className="primary"
              onClick={() => window.location.reload()}
            >
              Retry
            </button>
          </>
        ) : (
          <p role="status">Opening workspace…</p>
        )}
      </main>
    );
  }

  const session = current.session;
  const onboarded = current.preferences.onboardingComplete;
  const heading = view === "workspace" ? "Session workspace" : "Settings";
  const showRecovery = current.recoveryNotice !== null;

  return (
    <div className="shell">
      <a className="skip-link" href="#workspace">
        Skip to workspace
      </a>
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">
            ✦
          </span>
          <span>ShadowMind</span>
        </div>
        <nav aria-label="Main navigation">
          <button
            aria-current={view === "workspace" ? "page" : undefined}
            onClick={() => setView("workspace")}
          >
            <span aria-hidden="true" className="nav-icon">
              ▦
            </span>
            Workspace
          </button>
          <button
            aria-current={view === "settings" ? "page" : undefined}
            onClick={() => setView("settings")}
          >
            <span aria-hidden="true" className="nav-icon">
              ⚙
            </span>
            Settings
          </button>
        </nav>
        <div className="sidebar-note">
          <span className="status-dot" aria-hidden="true" />
          <div>
            <strong>Local demo</strong>
            <small>Synthetic inputs only</small>
          </div>
        </div>
      </aside>
      <main id="workspace" className="content" tabIndex={-1}>
        <header className="page-header">
          <div>
            <h1>{heading}</h1>
            <p className="muted">
              {view === "workspace"
                ? "Configure and try a local practice session."
                : "Manage appearance and view processing permissions."}
            </p>
          </div>
          <span
            className="badge session-badge"
            data-active={session.phase === "active"}
            role="status"
          >
            <span className="status-dot" aria-hidden="true" />
            {phaseLabels[session.phase]}
          </span>
        </header>
        {showRecovery && (
          <p className="notice" role="status">
            Saved settings could not be loaded. ShadowMind opened with defaults.
          </p>
        )}
        {error && (
          <p className="notice" role="alert">
            {error}
          </p>
        )}
        {view === "settings" ? (
          <Settings
            key={
              current.preferences.appearance.theme +
              current.preferences.appearance.textScale
            }
            appearance={current.preferences.appearance}
            busy={busy}
            save={(next) => run(() => savePreferences(next, false))}
          />
        ) : !onboarded ? (
          <section
            className="panel onboarding"
            aria-labelledby="welcome-heading"
          >
            <h2 id="welcome-heading">Set up your workspace</h2>
            <p className="muted">
              Start with bundled examples. Screen access, microphone access, and
              remote processing are off.
            </p>
            <button
              className="primary"
              disabled={busy}
              onClick={() =>
                void run(() =>
                  savePreferences(current.preferences.appearance, true),
                )
              }
            >
              Continue to workspace
            </button>
          </section>
        ) : (
          <Workspace
            session={session}
            busy={busy}
            send={(action: SessionAction) => run(() => dispatchSession(action))}
          />
        )}
      </main>
    </div>
  );
}
