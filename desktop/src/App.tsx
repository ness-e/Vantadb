import { useState } from "react";
import "./App.css";
import { useConnectionState } from "./hooks/useConnectionState";
import WorkspaceShell from "./components/layout/WorkspaceShell";

// App es un contenedor fino: estado del backend (useConnectionState), tema y
// notice/error. Toda la estructura de workspace (sidebar/topbar/superficies/
// inspector) vive en WorkspaceShell (VS-03). App.css se conserva: los paneles
// legacy reubicados como superficies dependen de .panel/.metrics-grid/.kpi-grid/
// .conn-list/.explorer-table, etc.
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
    <WorkspaceShell
      state={state}
      actions={actions}
      notice={notice}
      onNotice={setNotice}
      onDismissNotice={() => setNotice(null)}
      onError={reportError}
      dark={dark}
      onToggleTheme={toggleTheme}
    />
  );
}

export default App;