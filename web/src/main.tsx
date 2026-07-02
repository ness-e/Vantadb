// CRITICAL: GSAP plugins MUST be registered before React renders any component.
// Rollup's module execution order in production is non-deterministic for dynamic imports,
// so placing this here (the true app entry point) ensures guaranteed initialization.
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { TextPlugin } from "gsap/TextPlugin";
import { useGSAP } from "@gsap/react";
gsap.registerPlugin(ScrollTrigger, TextPlugin, useGSAP);

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
