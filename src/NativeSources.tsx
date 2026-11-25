import { useState } from "react";
import {
  errorMessage,
  listAudioSources,
  type AudioChoice,
  type NativeKind,
  type NativeStatus,
  type SessionSnapshot,
} from "./bridge";

export const inputLabels: Record<NativeStatus, string> = {
  idle: "Off",
  connecting: "Connecting",
  disconnecting: "Releasing source",
  receiving: "Receiving locally",
  unavailable: "Unavailable",
  denied: "Permission ended",
  suspended: "Suspended",
};

export const inputRunning = (status: NativeStatus) =>
  status === "connecting" ||
  status === "receiving" ||
  status === "disconnecting";

export default function NativeSources({
  session,
  busy,
  start,
}: {
  session: SessionSnapshot;
  busy: boolean;
  start: (
    kind: NativeKind,
    confirmed: boolean,
    sourceId?: string,
  ) => Promise<void>;
}) {
  const [screenConsent, setScreenConsent] = useState(false);
  const [audioConsent, setAudioConsent] = useState(false);
  const [choices, setChoices] = useState<AudioChoice[] | null>(null);
  const [selected, select] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const id = session.id;
  if (!id) return null;

  async function refresh() {
    if (!id) return;
    setChoices(null);
    select("");
    setLoading(true);
    setError("");
    try {
      setChoices(await listAudioSources(id));
    } catch (failure) {
      setError(errorMessage(failure));
    } finally {
      setLoading(false);
    }
  }

  return (
    <section className="panel image-panel" aria-labelledby="native-sources">
      <h2 id="native-sources">Live inputs</h2>
      <p className="muted">
        Use only sources you are permitted to share. Input stays on this device.
      </p>
      <div className="settings-grid native-grid">
        {session.mode !== "audio" && (
          <div>
            <div className="section-heading">
              <h3>Screen window</h3>
              <span className="badge" role="status">
                {inputLabels[session.screenStatus]}
              </span>
            </div>
            <label className="confirm">
              <input
                type="checkbox"
                checked={screenConsent}
                disabled={inputRunning(session.screenStatus)}
                onChange={(event) =>
                  setScreenConsent(event.currentTarget.checked)
                }
              />
              I have permission to share the window I select.
            </label>
            <div className="actions">
              <button
                className="secondary"
                disabled={
                  !screenConsent || busy || inputRunning(session.screenStatus)
                }
                onClick={() => {
                  void start("screen", screenConsent);
                  setScreenConsent(false);
                }}
              >
                Choose window
              </button>
            </div>
          </div>
        )}
        {session.mode !== "screen" && (
          <div>
            <div className="section-heading">
              <h3>Audio source</h3>
              <span className="badge" role="status">
                {inputLabels[session.audioStatus]}
              </span>
            </div>
            <div className="settings-form">
              <button
                className="secondary"
                disabled={loading || busy || inputRunning(session.audioStatus)}
                onClick={() => void refresh()}
              >
                {loading ? "Finding sources…" : "Find audio sources"}
              </button>
              {choices?.length === 0 && (
                <p role="status">
                  No selectable PipeWire audio source was found.
                </p>
              )}
              {choices && choices.length > 0 && (
                <fieldset className="audio-choices">
                  <legend>Select audio source</legend>
                  {choices.map((choice) => (
                    <label className="audio-choice" key={choice.id}>
                      <input
                        type="radio"
                        name="audio-source"
                        checked={selected === choice.id}
                        onChange={() => select(choice.id)}
                      />
                      <span>{choice.label}</span>
                    </label>
                  ))}
                </fieldset>
              )}
            </div>
            <label className="confirm">
              <input
                type="checkbox"
                checked={audioConsent}
                disabled={inputRunning(session.audioStatus)}
                onChange={(event) =>
                  setAudioConsent(event.currentTarget.checked)
                }
              />
              I have permission to use this audio source, including participant
              consent where required.
            </label>
            <div className="actions">
              <button
                className="secondary"
                disabled={
                  !audioConsent ||
                  !selected ||
                  loading ||
                  busy ||
                  inputRunning(session.audioStatus)
                }
                onClick={() => {
                  void start("audio", audioConsent, selected);
                  setAudioConsent(false);
                  setChoices(null);
                  select("");
                }}
              >
                Start audio input
              </button>
            </div>
            {error && (
              <p className="notice" role="alert">
                {error}
              </p>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
