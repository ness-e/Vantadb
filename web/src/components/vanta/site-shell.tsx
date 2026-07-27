"use client";

import { type ReactNode, useCallback } from "react";
import { usePathname } from "next/navigation";
import { SiteNavbar } from "./site-navbar";
import { Footer } from "./footer";
import { ScrollProgress } from "./scroll-progress";
import { CommandPalette } from "./command-palette";
import { BackToTop } from "./back-to-top";
import { ShortcutOverlay, ShortcutHintButton } from "./shortcut-overlay";
import { PageTransition } from "./page-transition";
import { EasterEgg } from "./easter-egg";
import { useVantaNavigate } from "@/hooks/use-vanta-navigate";

/**
 * SiteShell — shared layout shell wrapping all App Router pages.
 *
 * Replaces the old SPA page.tsx logic:
 *  - Derives navigation from useVantaNavigate (router.push)
 *  - PageTransition keyed by pathname (real URL changes)
 *  - Renders: ScrollProgress + Navbar + main(children) + Footer + global modals
 *
 * Used in src/app/layout.tsx as the root layout wrapper.
 */
export function SiteShell({ children }: { children: ReactNode }) {
  const navigate = useVantaNavigate();
  const pathname = usePathname();

  // Open shortcut overlay via a synthetic event when the navbar "?" is clicked
  const openShortcuts = useCallback(() => {
    if (typeof window === "undefined") return;
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "?", bubbles: true }));
  }, []);

  return (
    <div className="flex min-h-[100dvh] flex-col overflow-x-hidden bg-[#FBF9F5] text-black">
      <ScrollProgress />
      <SiteNavbar
        onNavigate={navigate}
        extraActions={<ShortcutHintButton onClick={openShortcuts} />}
      />
      <main id="main-content" className="flex-1">
        <PageTransition viewKey={pathname ?? "home"}>{children}</PageTransition>
      </main>
      <Footer onNavigate={navigate} />
      <BackToTop />
      <CommandPalette onNavigate={navigate} />
      <ShortcutOverlay onNavigate={navigate} />
      <EasterEgg />
    </div>
  );
}
