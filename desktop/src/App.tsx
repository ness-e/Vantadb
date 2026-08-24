import { useEffect, useState } from "react";
import "./App.css";
import { isEmbedded } from "./transport";
import { useConnectionState } from "./hooks/useConnectionState";
import WorkspaceShell from "./components/layout/WorkspaceShell";
import { TitleBar } from "./components/layout/TitleBar";
import { SplashScreen } from "./components/layout/SplashScreen";

// App es un contenedor fino: estado del backend (useConnectionState), tema y
// notice/error. Toda la estructura de workspace (sidebar/topbar/superficies/
// inspector) vive en WorkspaceShell (VS-03). App.css se conserva: los paneles
// legacy reubicados como superficies dependen de .panel/.metrics-grid/.kpi-grid/
// .conn-list/.explorer-table, etc.
function App() {
  const [state, actions] = useConnectionState();
  const [notice, setNotice] = useState<string | null>(null);
  // FIND-22: initial state mirrors what main.tsx already applied (stored
  // choice or OS preference); toggle records an explicit manual choice which
  // stops OS-following.
  const [dark, setDark] = useState(() =>
    document.documentElement.classList.contains("dark"),
  );

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

  // FIND-23: cold-start splash, once per app session. Connection init runs in
  // parallel — the splash never blocks it.
  const [splash, setSplash] = useState(true);

  return (
    // FIND-19: custom window chrome only on the Tauri build; web builds keep
    // native browser chrome.
    <div className="flex h-screen flex-col overflow-hidden">
      {splash && <SplashScreen onDismiss={() => setSplash(false)} />}
      {!isEmbedded && <TitleBar />}
      <WorkspaceShell
        // WEB-05: web build hides the Tauri-only connection selector; the
        // embedded HTTP connection is active by default (useConnectionState).
        embedded={isEmbedded}
        state={state}
        actions={actions}
        notice={notice}
        onNotice={setNotice}
        onDismissNotice={() => setNotice(null)}
        onError={reportError}
        dark={dark}
        onToggleTheme={toggleTheme}
      />
    </div>
  );
}

export default App;