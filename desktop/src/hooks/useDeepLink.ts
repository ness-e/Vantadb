// useDeepLink (VS-16): consumes `vanta://` deep links — both the ones buffered
// while the app was starting (vanta_deep_link_take) and live arrivals emitted
// by Rust as the `vanta-deep-link` event (payload: string[] of raw URLs).
//
// Parsing is delegated to `parseVantaUrl` (strict: any non-`vanta://` input
// returns null). The caller decides how to navigate — this hook only surfaces
// parsed links.
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { isEmbedded } from "../transport";
import { DEEP_LINK_EVENT, parseVantaUrl, takeDeepLink, type VantaDeepLink } from "../vanta";

export function useDeepLink(onLink: (link: VantaDeepLink) => void): void {
  useEffect(() => {
    // WEB-05: deep links are an OS-scheme/IPC feature — they cannot arrive in
    // the web build. Guard runtime (no dynamic import needed: @tauri-apps/api
    // bundles fine, only calling `listen` outside Tauri would reject).
    if (isEmbedded) {
      console.info("[vanta] deep links no disponibles en modo web (Tauri-only)");
      return;
    }

    let disposed = false;

    async function handle(raw: string) {
      const parsed = parseVantaUrl(raw);
      if (parsed) onLink(parsed);
    }

    // Links that arrived before the frontend mounted (incl. the startup URL).
    takeDeepLink()
      .then((urls) => {
        if (disposed) return;
        for (const u of urls) void handle(u);
      })
      .catch((err) => {
        // Best-effort: a failing take must not break the app shell, but a
        // silent drop would hide lost deep links — surface it to console.
        console.warn("[vanta] vanta_deep_link_take failed:", err);
      });

    // Live arrivals while the app runs (single-instance callback → Rust emit).
    const unlisten = listen<string[]>(DEEP_LINK_EVENT, (event) => {
      for (const u of event.payload) void handle(u);
    });

    return () => {
      disposed = true;
      void unlisten.then((fn) => fn());
    };
  }, [onLink]);
}