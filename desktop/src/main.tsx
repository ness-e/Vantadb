import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

// Theme init BEFORE mount — default light (D7); only a stored "dark" opts in.
if (localStorage.getItem("vanta-theme") === "dark") {
  document.documentElement.classList.add("dark");
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
