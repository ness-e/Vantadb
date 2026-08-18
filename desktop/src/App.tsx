import { useState } from "react";
import "./App.css";
import { useConnectionState } from "./hooks/useConnectionState";
import ConnectionPanel from "./components/ConnectionPanel";
import IngestForm from "./components/IngestForm";
import SearchBar from "./components/SearchBar";
import KpiCards from "./components/KpiCards";
import MetricsGrid from "./components/MetricsGrid";
import DataExplorer from "./components/DataExplorer";
import SopPanel from "./components/SopPanel";
import ProcessPanel from "./components/ProcessPanel";
import ExportPanel from "./components/ExportPanel";

function App() {
  const [state, actions] = useConnectionState();
  const [notice, setNotice] = useState<string | null>(null);
  const [dark, setDark] = useState(() => document.documentElement.classList.contains("dark"));

  function toggleTheme() {
    const next = !dark;
    setDark(next);
    document.documentElement.classList.toggle("dark", next);
    localStorage.setItem("vanta-theme", next ? "dark" : "light");
  }

  function reportError(msg: string) {
    actions.clearError();
    setNotice(msg ?? "Operation failed.");
  }

  return (
    <main className="app">
      <header className="app-head">
        <h1>VantaDB Desktop</h1>
        <span className="muted">Embedded memory, local-first</span>
        <button
          type="button"
          className="press"
          onClick={toggleTheme}
          style={{ marginLeft: "auto" }}
          aria-label={dark ? "Switch to light mode" : "Switch to dark mode"}
        >
          {dark ? "☀ LIGHT" : "☾ DARK"}
        </button>
      </header>

      {notice && (
        <div className="error" role="alert" onClick={() => setNotice(null)}>
          {notice}
        </div>
      )}

      <MetricsGrid
        health={state.health}
        healthStatus={state.healthStatus}
        activeName={state.active ? state.active.name : null}
      />

      <KpiCards />

      <ExportPanel />

      <SopPanel />

      <div className="grid">
        <ConnectionPanel
          connections={state.connections}
          activeId={state.activeId}
          health={state.health}
          healthStatus={state.healthStatus}
          busy={state.busy}
          onConnectNative={actions.connectNativePath}
          onDisconnect={actions.disconnectId}
          onActivate={actions.activate}
          onProbeHealth={actions.probeHealth}
        />
        <IngestForm onDone={(ids) => setNotice(`Stored ${ids.length} record(s).`)} runError={reportError} />
        <SearchBar busy={state.busy} runError={reportError} />
      </div>

      <DataExplorer active={!!state.active} busy={state.busy} runError={reportError} />

      <ProcessPanel
        connections={state.connections}
        activeId={state.activeId}
        onShutdown={actions.disconnectId}
        onActivate={actions.activate}
      />

      <footer className="muted">
        {state.active
          ? `Active: ${state.active.name} (${state.active.via})`
          : "No active backend — connect one to enable ingest/search."}
      </footer>
    </main>
  );
}

export default App;