// Orchestrates connection lifecycle + health + active-connection state.
// Drives DActions purely through the typed bridge (src/vanta.ts).
import { useCallback, useEffect, useState } from "react";
import { isEmbedded } from "../transport";
import {
  connectNative,
  ConnectionInfo,
  disconnect,
  HealthReport,
  health,
  listConnections,
  setActive,
  vantaErrorMessage,
} from "../vanta";

export interface VantaState {
  connections: ConnectionInfo[];
  activeId: string | null;
  active: ConnectionInfo | null;
  health: HealthReport | null;
  healthStatus: "ok" | "warn" | "err" | "idle";
  busy: boolean;
  error: string | null;
}

const initial: VantaState = {
  connections: [],
  activeId: null,
  active: null,
  health: null,
  healthStatus: "idle",
  busy: false,
  error: null,
};

function healthStatus(r: HealthReport | null): VantaState["healthStatus"] {
  if (!r) return "idle";
  if (r.status === "healthy") return "ok";
  if (r.status === "degraded") return "warn";
  return "err";
}

export interface ConnectionActions {
  refresh: () => Promise<void>;
  probeHealth: () => Promise<void>;
  connectNativePath: (path: string) => Promise<string | null>;
  disconnectId: (id: string) => Promise<void>;
  activate: (id: string) => Promise<void>;
  clearError: () => void;
}

export function useConnectionState(): [VantaState, ConnectionActions] {
  const [state, setState] = useState<VantaState>(initial);

  const patch = useCallback((p: Partial<VantaState>) => {
    setState((s) => ({ ...s, ...p }));
  }, []);

  const refresh = useCallback(async () => {
    try {
      // WEB-05: in embedded (web) mode there is no multi-connection manager —
      // the app talks to the embedded server directly, so the connection list
      // is one implicit HTTP connection ("modo embebido = HTTP activo por
      // defecto"). listConnections() throws on the HTTP transport (unsupported).
      const pairs: [string, ConnectionInfo][] = isEmbedded
        ? [
            [
              "embedded",
              {
                id: "embedded",
                name: "embedded",
                via: "http",
                status: "connected",
                description: "servidor embebido (HTTP)",
              },
            ],
          ]
        : await listConnections();
      const conns = pairs.map(([, info]) => info);
      setState((s) => {
        const activeId =
          s.activeId && conns.some((c) => c.id === s.activeId)
            ? s.activeId
            : conns.length
              ? conns[conns.length - 1].id
              : null;
        return {
          ...s,
          connections: conns,
          activeId,
          active: conns.find((c) => c.id === activeId) ?? null,
          error: null,
        };
      });
    } catch (e) {
      patch({ error: vantaErrorMessage(e) });
    }
  }, [patch]);

  const probeHealth = useCallback(async () => {
    patch({ busy: true });
    try {
      const r = await health();
      patch({ health: r, healthStatus: healthStatus(r), error: null });
    } catch (e) {
      patch({ healthStatus: "err", error: vantaErrorMessage(e) });
    } finally {
      patch({ busy: false });
    }
  }, [patch]);

  const connectNativePath = useCallback(
    async (path: string): Promise<string | null> => {
      patch({ busy: true });
      try {
        const info = await connectNative(path);
        await refresh();
        await setActive(info.id);
        patch({ activeId: info.id, error: null });
        return info.id;
      } catch (e) {
        patch({ error: vantaErrorMessage(e) });
        return null;
      } finally {
        patch({ busy: false });
      }
    },
    [patch, refresh],
  );

  const disconnectId = useCallback(
    async (id: string) => {
      patch({ busy: true });
      try {
        await disconnect(id);
        await refresh();
      } catch (e) {
        patch({ error: vantaErrorMessage(e) });
      } finally {
        patch({ busy: false });
      }
    },
    [patch, refresh],
  );

  const activate = useCallback(
    async (id: string) => {
      try {
        await setActive(id);
        patch({ activeId: id });
      } catch (e) {
        patch({ error: vantaErrorMessage(e) });
      }
    },
    [patch],
  );

  // Initial refresh + health probe.
  useEffect(() => {
    refresh();
    probeHealth();
  }, [refresh, probeHealth]);

  return [
    state,
    {
      refresh,
      probeHealth,
      connectNativePath,
      disconnectId,
      activate,
      clearError: () => patch({ error: null }),
    },
  ];
}