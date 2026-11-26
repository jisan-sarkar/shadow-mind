import { useState } from "react";
import type {
  InputMode,
  Phase,
  SessionAction,
  SessionSnapshot,
} from "./bridge";

const modes: { value: InputMode; label: string; detail: string }[] = [
  { value: "screen", label: "Screen", detail: "Visual context" },
  { value: "audio", label: "Audio", detail: "Spoken context" },
  { value: "both", label: "Both", detail: "Visual and spoken context" },
];

export const phaseLabels: Record<Phase, string> = {
  idle: "Idle",
  configuring: "Configuring",
  active: "Demo active",
  paused: "Paused",
  stopping: "Stopping",
  stopped: "Stopped",
};

const fixtureStatus = {
  ready: "Ready",
  processing: "Processing",
  complete: "Complete",
  cancelled: "Cancelled",
  failed: "Unavailable",
} as const;

interface Props {
  session: SessionSnapshot;
  busy: boolean;
  send: (action: SessionAction) => Promise<void>;
}

export default function Workspace({ session, busy, send }: Props) {
  const [mode, setMode] = useState<InputMode>("screen");
  const [confirmedSession, confirm] = useState<string | null>(null);
  const canConfigure = session.phase === "idle" || session.phase === "stopped";
  const { id, phase } = session;
  const selected = modes.find(
    (choice) => choice.value === (canConfigure ? mode : session.mode),
  );

  return (
    <div className="workspace-grid">
      <section
        className="panel session-panel"
        aria-labelledby="session-heading"
      >
        <h2 id="session-heading">Session setup</h2>
        <p className="muted">Choose an input mode for your demo session.</p>
        <fieldset disabled={!canConfigure || busy} className="mode-list">
          <legend className="sr-only">Input mode</legend>
          {modes.map((choice) => (
            <label className="mode-choice" key={choice.value}>
              <input
                type="radio"
                name="inputMode"
                value={choice.value}
                checked={selected?.value === choice.value}
                onChange={() => setMode(choice.value)}
              />
              <span>
                <strong>{choice.label}</strong>
                <small>{choice.detail}</small>
              </span>
            </label>
          ))}
        </fieldset>
        <dl className="session-details">
          <div>
            <dt>Source</dt>
            <dd>Bundled fixtures</dd>
          </div>
          <div>
            <dt>Provider</dt>
            <dd>Local mock · available</dd>
          </div>
        </dl>
        {phase === "configuring" && id && (
          <label className="confirm">
            <input
              type="checkbox"
              checked={confirmedSession === id}
              disabled={busy}
              onChange={(event) =>
                confirm(event.currentTarget.checked ? id : null)
              }
            />
            Use bundled synthetic inputs for this session.
          </label>
        )}
        <div className="actions">
          {canConfigure && (
            <button
              className="primary"
              disabled={busy}
              onClick={() => void send({ type: "configure", mode })}
            >
              Configure session
            </button>
          )}
          {id && phase === "configuring" && (
            <button
              className="primary"
              disabled={busy || confirmedSession !== id}
              onClick={() =>
                void send({
                  type: "start",
                  sessionId: id,
                  confirmFixtures: confirmedSession === id,
                })
              }
            >
              Start demo
            </button>
          )}
          {id && phase === "active" && (
            <button
              className="secondary"
              disabled={busy}
              onClick={() => void send({ type: "pause", sessionId: id })}
            >
              Pause
            </button>
          )}
          {id && phase === "paused" && (
            <button
              className="primary"
              disabled={busy}
              onClick={() => void send({ type: "resume", sessionId: id })}
            >
              Resume
            </button>
          )}
          {id && !canConfigure && phase !== "stopping" && (
            <button
              className="quiet"
              disabled={busy}
              onClick={() => void send({ type: "stop", sessionId: id })}
            >
              {phase === "configuring" ? "Cancel setup" : "Stop session"}
            </button>
          )}
        </div>
      </section>

      <section
        className="panel answer-panel"
        aria-labelledby="response-heading"
      >
        <div className="section-heading">
          <h2 id="response-heading">Demo response</h2>
          <span role="status" className="badge">
            {fixtureStatus[session.fixtureStatus]}
          </span>
        </div>
        {session.fixture ? (
          <article className="response-content">
            <span className="response-label">Fixture question</span>
            <h3>{session.fixture.question}</h3>
            <span className="response-label">Mock answer</span>
            <p>{session.fixture.answer}</p>
          </article>
        ) : (
          <div className="empty-response">
            <span className="empty-mark" aria-hidden="true">
              <i />
              <i />
              <i />
            </span>
            <h3>
              {session.fixtureStatus === "processing"
                ? "Running the fixture"
                : "No response yet"}
            </h3>
            <p className="muted">
              {session.fixtureStatus === "failed"
                ? "The bundled fixture could not be loaded."
                : session.fixtureStatus === "cancelled"
                  ? "The fixture request was cancelled."
                  : phase === "active"
                    ? "Run a bundled example to see a mock answer."
                    : "Start a demo session to run a bundled example."}
            </p>
          </div>
        )}
        {phase === "active" && id && (
          <div className="actions answer-actions">
            {session.fixtureStatus === "processing" ? (
              <button
                className="secondary"
                disabled={busy}
                onClick={() =>
                  void send({ type: "cancelFixture", sessionId: id })
                }
              >
                Cancel fixture
              </button>
            ) : (
              <button
                className="secondary"
                disabled={busy}
                onClick={() => void send({ type: "runFixture", sessionId: id })}
              >
                Run fixture
              </button>
            )}
          </div>
        )}
      </section>
    </div>
  );
}
