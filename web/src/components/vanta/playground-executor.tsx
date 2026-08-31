"use client";

import { useEffect, useRef, useCallback, useState, forwardRef } from "react";

const IFRAME_URL = "/playground-executor.html";

interface ExecuteRequest {
  type: "execute";
  code: string;
  requestId: number;
}

interface ExecuteResponse {
  type: "result";
  requestId: number;
  output: string[];
  error?: string;
}

interface ReadyMessage {
  type: "ready";
}

type IframeMessage = ExecuteResponse | ReadyMessage;

export const PlaygroundExecutor = forwardRef<HTMLIFrameElement, {
  onReady?: () => void;
  onResult: (output: string[], error?: string) => void;
}>(
  function PlaygroundExecutor({ onReady, onResult }, ref) {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const requestIdRef = useRef(0);
  const pendingCallbacksRef = useRef<Map<number, (result: { output: string[]; error?: string }) => void>>(new Map());
  const [isReady, setIsReady] = useState(false);

  // Expose iframe ref to parent via forwarded ref
  useEffect(() => {
    if (ref) {
      if (typeof ref === "function") {
        ref(iframeRef.current);
      } else {
        ref.current = iframeRef.current;
      }
    }
    return () => {
      if (ref) {
        if (typeof ref === "function") {
          ref(null);
        } else {
          ref.current = null;
        }
      }
    };
  }, [ref]);

  const handleMessage = useCallback((event: MessageEvent) => {
    const data = event.data as IframeMessage;
    if (!data || !data.type) return;

    if (data.type === "ready") {
      setIsReady(true);
      onReady?.();
      return;
    }

    if (data.type === "result") {
      const callback = pendingCallbacksRef.current.get(data.requestId);
      if (callback) {
        callback({ output: data.output, error: data.error });
        pendingCallbacksRef.current.delete(data.requestId);
      }
    }
  }, [onReady]);

  useEffect(() => {
    window.addEventListener("message", handleMessage);
    return () => window.removeEventListener("message", handleMessage);
  }, [handleMessage]);

  const execute = useCallback((code: string): Promise<{ output: string[]; error?: string }> => {
    return new Promise((resolve) => {
      const iframe = iframeRef.current;
      if (!iframe || !isReady) {
        resolve({ output: [], error: "Executor not ready" });
        return;
      }

      const requestId = ++requestIdRef.current;
      pendingCallbacksRef.current.set(requestId, resolve);

      iframe.contentWindow?.postMessage(
        { type: "execute", code, requestId } as ExecuteRequest,
        "*"
      );

      // Timeout fallback
      setTimeout(() => {
        if (pendingCallbacksRef.current.has(requestId)) {
          pendingCallbacksRef.current.delete(requestId);
          resolve({ output: [], error: "Execution timeout" });
        }
      }, 30000);
    });
  }, [isReady]);

  return (
    <iframe
      ref={ref}
      sandbox="allow-scripts allow-same-origin"
      src={IFRAME_URL}
      style={{ display: "none" }}
      title="VantaDB Playground Executor"
      aria-hidden="true"
    />
  );
});

export function usePlaygroundExecutor() {
  const executorRef = useRef<{ execute: (code: string) => Promise<{ output: string[]; error?: string }> } | null>(null);
  const [isReady, setIsReady] = useState(false);

  const setExecutorRef = useCallback((node: HTMLIFrameElement | null) => {
    if (node) {
      executorRef.current = {
        execute: (code: string): Promise<{ output: string[]; error?: string }> => {
          return new Promise((resolve) => {
            const requestId = Date.now() + Math.random();
            const handleMessage = (event: MessageEvent) => {
              const data = event.data;
              if (data && data.type === "result" && data.requestId === requestId) {
                window.removeEventListener("message", handleMessage);
                resolve({ output: data.output, error: data.error });
              }
            };
            window.addEventListener("message", handleMessage);
            node.contentWindow?.postMessage({ type: "execute", code, requestId }, "*");
            setTimeout(() => {
              window.removeEventListener("message", handleMessage);
              resolve({ output: [], error: "Execution timeout" });
            }, 30000);
          });
        },
      };
    }
  }, []);

  return { executorRef: setExecutorRef, isReady, execute: executorRef.current?.execute };
}