import { redirect } from "next/navigation";

/**
 * /demo — Redirects to /playground (the interactive code playground).
 * The demo and playground serve the same purpose: try VantaDB code in-browser.
 */
export default function DemoPage() {
  redirect("/playground");
}
