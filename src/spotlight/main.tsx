import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import "./spotlight.css";

function SpotlightApp() {
  return (
    <main className="spotlight-shell">
      <section className="spotlight-card">
        <p className="spotlight-label">Spotlight</p>
        <h1 className="spotlight-title">Quick Search</h1>
        <input
          className="spotlight-input"
          type="text"
          // biome-ignore lint/a11y/noAutofocus: need autoFocus
          autoFocus
          placeholder="Type to search..."
          aria-label="Search"
        />
      </section>
    </main>
  );
}

ReactDOM.createRoot(document.getElementById("spotlight-root") as HTMLElement).render(
  <StrictMode>
    <SpotlightApp />
  </StrictMode>,
);
