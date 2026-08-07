import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [pingMsg, setPingMsg] = useState<string>("");

  async function ping() {
    // Calls the `ping` Rust command — proves the IPC bridge round-trips.
    // Source: https://tauri.app/develop/calling-rust/
    setPingMsg(await invoke<string>("ping"));
  }

  return (
    <main className="container">
      <h1>VantaDB Desktop</h1>

      <button type="button" onClick={ping}>
        Ping Rust
      </button>
      <p className="ping-result">
        {pingMsg ? `Rust replied: ${pingMsg}` : "Click to ping the Rust backend."}
      </p>
    </main>
  );
}

export default App;
