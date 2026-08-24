import { notFound } from "next/navigation";

// Unmatched URLs land here (catch-all) and delegate to the nearest
// not-found boundary: app/not-found.tsx keeps the styled 404 UI.
// Source: https://nextjs.org/docs/app/api-reference/functions/not-found
export default function CatchAllNotFoundPage() {
  notFound();
}
