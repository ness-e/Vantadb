import { useEffect, useRef, useState } from "react";

export function EngineWALSimulator() {
  const [logs, setLogs] = useState<string[]>([
    "[00:00:01] System boot initialized",
    "[00:00:02] Storage engine opened at path ./agent_memory",
    "[00:00:03] Replaying WAL logs... 0 transactions found",
    "[00:00:04] Database state: READY",
  ]);
  const [engineState, setEngineState] = useState<"ready" | "crashed" | "recovering">("ready");
  const consoleEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (consoleEndRef.current) {
      consoleEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, []);

  const triggerCrash = () => {
    if (engineState !== "ready") return;
    setEngineState("crashed");
    setLogs((prev) => [
      ...prev,
      "[00:02:15] put txn: namespace=memories, key=conv-88",
      "[00:02:16] wal: writing page log CRC32C=0xab12de",
      "!!! CRITICAL FAILURE: PROCESS TERMINATED OUTSIDE CLEAN DISCONNECT !!!",
      "STATUS: OFFLINE",
    ]);
  };

  const recoverFromWAL = () => {
    if (engineState !== "crashed") return;
    setEngineState("recovering");
    setLogs((prev) => [
      ...prev,
      "[00:03:01] Database reopened. Initializing WAL scan...",
      "[00:03:02] WAL found. Unflushed write at sector index 43",
      "[00:03:03] Checking integrity: verifying CRC32C checksums...",
      "[00:03:04] Checksum 0xab12de OK. Syncing WAL entry 1/1",
      "[00:03:05] WAL sync finished. Rebuilding transient HNSW indexes",
      "[00:03:06] State restored in 0.4ms. 1 transaction recovered.",
      "STATUS: READY",
    ]);
    setTimeout(() => {
      setEngineState("ready");
    }, 1200);
  };

  const getLogLevel = (log: string) => {
    if (log.includes("!!!")) return "error";
    if (log.includes("READY") || log.includes("OK")) return "ok";
    if (log.includes("RECOVER") || log.includes("Syncing")) return "warn";
    return "info";
  };

  return (
    <div className="nc-engine-recorder">
      <div className="nc-engine-recorder-header">
        <div className="nc-engine-recorder-status">
          <span className="nc-engine-recorder-dot" data-state={engineState} />
          <span className="nc-engine-recorder-status-text">
            STATUS: {engineState.toUpperCase()}
          </span>
        </div>
        <div className="nc-engine-recorder-actions">
          <button
            className="nb-btn nb-btn--ghost"
            onClick={triggerCrash}
            disabled={engineState !== "ready"}
          >
            CRASH ENGINE
          </button>
          <button className="nb-btn" onClick={recoverFromWAL} disabled={engineState !== "crashed"}>
            RECOVER FROM WAL
          </button>
        </div>
      </div>

      <div className="nc-engine-recorder-console">
        {logs.map((log) => (
          <div key={log} className="nc-engine-recorder-line" data-level={getLogLevel(log)}>
            {log}
          </div>
        ))}
        <div ref={consoleEndRef} />
      </div>
    </div>
  );
}
