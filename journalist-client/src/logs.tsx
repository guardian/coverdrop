import React from "react";
import { EuiFlyout, EuiProvider } from "@elastic/eui";
import "./euiIconsWorkAround";
import { createRoot } from "react-dom/client";
import { Logs } from "./components/Logs.tsx";

createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <EuiProvider colorMode="dark">
      <EuiFlyout size="100%" hideCloseButton={true} onClose={() => {}}>
        <Logs />
      </EuiFlyout>
    </EuiProvider>
  </React.StrictMode>,
);
