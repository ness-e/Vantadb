import { useState } from "react";
import "./App.css";
import { useConnectionState } from "./hooks/useConnectionState";
import ConnectionPanel from "./components/ConnectionPanel";
import IngestForm from "./components/IngestForm";
import SearchBar from "./components/SearchBar";

function App() {
  const [state, actions] = useConnectionState();
  const [notice, setNotice] = useState<string | null>(null);

  function reportError(msg: string) {
    actions.clearError();
    setNotice(msg ?? "Operation failed.");
  }

  return (
    <main className="app">
      <header className="app-head">
        <h1>VantaDB Desktop</h1>
        <span className="muted">Embedded memory, local-first</span>
      </header>

      {notice && (
        <div className="error" role="alert" onClick={() => setNotice(null)}>
          {notice}
        </div>
      )}

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

      <footer className="muted">
        {state.active
          ? `Active: ${state.active.name} (${state.active.via})`
          : "No active backend — connect one to enable ingest/search."}
      </footer>
    </main>
  );
}

export default App;