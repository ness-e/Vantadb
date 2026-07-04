import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/docs-api")({
  beforeLoad: () => {
    throw redirect({ to: "/docs" });
  },
});
