import { redirect } from "next/navigation";

// /docs-api — Redirects to /docs (matches diseño2 behavior).
// Server component: no "use client" needed.
export default function DocsApiPage() {
  redirect("/docs");
}
