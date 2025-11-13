import React from "react";
import App from "./App";
import { EuiProvider } from "@elastic/eui";
import "./euiIconsWorkAround";
import { createRoot } from "react-dom/client";

createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <EuiProvider colorMode="light">
      <App />
    </EuiProvider>
  </React.StrictMode>,
);
