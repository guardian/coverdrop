import React from "react";
import App from "./App";
import { EuiProvider } from "@elastic/eui";
import "./euiIconsWorkAround";
import { createRoot } from "react-dom/client";
import { ErrorBoundary } from "./ErrorBoundary.tsx";

createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <EuiProvider colorMode="light">
      <ErrorBoundary>
        <App />
      </ErrorBoundary>
    </EuiProvider>
  </React.StrictMode>,
);
