import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

// Theme init BEFORE mount (FIND-22): stored manual choice wins; otherwise
// follow the OS preference. "manual" flag marks explicit user choice — while
// absent, live OS changes propagate.
const stored = localStorage.getItem("vanta-theme");
if (stored === "dark" || stored === "light") {
  document.documentElement.classList.toggle("dark", stored === "dark");
} else {
  const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  document.documentElement.classList.toggle("dark", prefersDark);
  // Follow live OS changes until the user picks manually.
  window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", (e) => {
      document.documentElement.classList.toggle("dark", e.matches);
    });
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
