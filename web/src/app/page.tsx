"use client";

import { HomeView } from "@/components/vanta/home-view";
import { useVantaNavigate } from "@/hooks/use-vanta-navigate";

/**
 * Home route (/) — renders the 11-section consolidated Home.
 * Navigation is handled by useVantaNavigate (router.push to /benchmarks, /docs).
 */
export default function HomePage() {
  const navigate = useVantaNavigate();
  return <HomeView onNavigate={navigate} />;
}
