import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { initWebVitals } from "./monitoring/webVitals";
import "./index.css";
import "./styles/tokens.css";

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("Root element #root not found");

ReactDOM.createRoot(rootEl).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

initWebVitals();
