import { useState } from "react";
import { chooseImage, errorMessage, type ImageMetadata } from "./bridge";

export default function ImageInspection({ sessionId }: { sessionId: string }) {
  const [authorized, authorize] = useState(false);
  const [pending, setPending] = useState(false);
  const [image, setImage] = useState<ImageMetadata | null>(null);
  const [error, setError] = useState("");

  async function inspect() {
    if (!authorized || pending) return;
    setPending(true);
    setImage(null);
    setError("");
    try {
      setImage(await chooseImage(sessionId, authorized));
    } catch (failure) {
      setError(errorMessage(failure));
    } finally {
      authorize(false);
      setPending(false);
    }
  }

  return (
    <section className="panel image-panel" aria-labelledby="image-heading">
      <h2 id="image-heading">Inspect an image</h2>
      <p className="muted">
        Check the format and dimensions of a local PNG or JPEG.
      </p>
      <label className="confirm">
        <input
          type="checkbox"
          checked={authorized}
          disabled={pending}
          onChange={(event) => authorize(event.currentTarget.checked)}
        />
        I have permission to inspect the image I select.
      </label>
      <div className="actions">
        <button
          className="secondary"
          disabled={!authorized || pending}
          onClick={() => void inspect()}
        >
          Choose image
        </button>
      </div>
      {pending && (
        <p className="muted" role="status">
          Waiting for the file chooser…
        </p>
      )}
      {error && (
        <p className="notice" role="alert">
          {error}
        </p>
      )}
      {image && (
        <p className="image-result" role="status">
          Valid {image.format} · {image.width} × {image.height} pixels
        </p>
      )}
    </section>
  );
}
