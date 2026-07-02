// CRITICAL: GSAP plugins MUST be registered before React renders any component.
// Rollup's module execution order in production is non-deterministic for dynamic imports,
// so placing this here (the true app entry point) ensures guaranteed initialization.
import { __gsap_plugins_registered } from "./lib/gsap";
// Dummy use to ensure bundler keeps the import and side effects
if (!__gsap_plugins_registered) {
  console.warn("GSAP plugins may not have registered correctly");
}

import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "@tanstack/react-router";
import { getRouter } from "./router";

// Import CSS stylesheet
import "./styles/index.css";

const router = getRouter();

const rootElement = document.getElementById("root");

if (rootElement) {
  const root = ReactDOM.createRoot(rootElement);
  root.render(
    <React.StrictMode>
      <RouterProvider router={router} />
    </React.StrictMode>,
  );
}
