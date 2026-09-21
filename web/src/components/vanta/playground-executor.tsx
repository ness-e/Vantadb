"use client";

import { useEffect, useRef, useCallback, useImperativeHandle, forwardRef } from "react";

const IFRAME_URL = "/playground-executor.html";

export type PlaygroundExecutorHandle = {
  execute: (code: string) => Promise<{ output: string[]; error?: string }>;
  isReady: () => boolean;
};

type ExecuteRequest = { type: "execute"; code: string; requestId: number };
type ExecuteResponse = { type: "result"; requestId: number; output: string[]; error?: string };
type ReadyMessage = { type: "ready" };
type IframeMessage = ExecuteResponse | ReadyMessage;

/**
 * WEB-07 — Iframe sandbox para el playground.
 * Ejecuta snippets del usuario aislados del DOM principal.
 * sandbox="allow-scripts allow-same-origin": allow-scripts aísla DOM/storage del parent;
 * allow-same-origin requerido para que el iframe pueda fetch /vanta-wasm/* del mismo origen
 * (sin él el iframe tiene origen opaco y fetch sería cross-origin sin CORS). Sin allow-top-navigation,
 * allow-forms, allow-popups → no puede navegar, enviar forms, ni abrir popups.
 * Comunicación vía postMessage con validación de source.
 * // ponytail: allow-same-origin es techo conocido — si /vanta-wasm se sirve con CORS o se embebe
 * // vía blob/srcdoc, se puede reducir a allow-scripts solo.
 */
export const PlaygroundExecutor = forwardRef<PlaygroundExecutorHandle, { onReady?: () => void }>(
  function PlaygroundExecutor({ onReady }, ref) {
    const iframeRef = useRef<HTMLIFrameElement>(null);
    const requestIdRef = useRef(0);
    const pendingRef = useRef<Map<number, (r: { output: string[]; error?: string }) => void>>(new Map());
    const readyRef = useRef(false);

    const handleMessage = useCallback(
      (event: MessageEvent) => {
        // Validate source is our iframe to avoid accepting messages from other origins/frames
        const iframe = iframeRef.current;
        if (iframe?.contentWindow && event.source !== iframe.contentWindow) return;
        const data = event.data as IframeMessage;
        if (!data || typeof data !== "object" || !("type" in data)) return;
        if (data.type === "ready") {
          readyRef.current = true;
          onReady?.();
          return;
        }
        if (data.type === "result") {
          const cb = pendingRef.current.get(data.requestId);
          if (cb) {
            cb({ output: data.output, error: data.error });
            pendingRef.current.delete(data.requestId);
          }
        }
      },
      [onReady],
    );

    useEffect(() => {
      window.addEventListener("message", handleMessage);
      return () => window.removeEventListener("message", handleMessage);
    }, [handleMessage]);

    // Ping retry: covers race where iframe posted ready before listener attached
    useEffect(() => {
      if (readyRef.current) return;
      let attempts = 0;
      const id = setInterval(() => {
        if (readyRef.current || attempts > 20) {
          clearInterval(id);
          return;
        }
        attempts += 1;
        iframeRef.current?.contentWindow?.postMessage({ type: "ping" }, "*");
      }, 500);
      return () => clearInterval(id);
    }, []);

    const handleIframeLoad = useCallback(() => {
      // Proactively ping after iframe load to trigger ready resend
      setTimeout(() => iframeRef.current?.contentWindow?.postMessage({ type: "ping" }, "*"), 100);
      setTimeout(() => iframeRef.current?.contentWindow?.postMessage({ type: "ping" }, "*"), 600);
    }, []);

    const execute = useCallback((code: string): Promise<{ output: string[]; error?: string }> => {
      return new Promise((resolve) => {
        const iframe = iframeRef.current;
        if (!iframe?.contentWindow || !readyRef.current) {
          resolve({ output: [], error: "Executor not ready — iframe aún no cargó" });
          return;
        }
        const requestId = ++requestIdRef.current;
        pendingRef.current.set(requestId, resolve);
        iframe.contentWindow.postMessage({ type: "execute", code, requestId } as ExecuteRequest, "*");
        setTimeout(() => {
          if (pendingRef.current.has(requestId)) {
            pendingRef.current.delete(requestId);
            resolve({ output: [], error: "Execution timeout (30s)" });
          }
        }, 30000);
      });
    }, []);

    useImperativeHandle(ref, () => ({ execute, isReady: () => readyRef.current }), [execute]);

    return (
      <iframe
        ref={iframeRef}
        sandbox="allow-scripts allow-same-origin"
        src={IFRAME_URL}
        style={{ display: "none" }}
        title="VantaDB Playground Executor"
        aria-hidden="true"
        tabIndex={-1}
        onLoad={handleIframeLoad}
      />
    );
  },
);
