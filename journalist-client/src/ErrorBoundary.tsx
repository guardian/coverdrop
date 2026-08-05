import React from "react";
import { EuiButton, EuiText } from "@elastic/eui";

type Props = {
  children: React.ReactNode;
};

type State = {
  error?: Error;
};

export class ErrorBoundary extends React.Component<Props, State> {
  public state: State = {};

  public static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  public componentDidCatch(error: Error, errorInfo: React.ErrorInfo): void {
    console.error("Uncaught render error:", error, errorInfo);
  }

  private handleReload = (): void => {
    window.location.reload();
  };

  public render(): React.ReactNode {
    if (this.state.error) {
      return (
        <div
          style={{
            minHeight: "100vh",
            display: "grid",
            placeItems: "center",
            padding: 24,
            background: "#f5f7fa",
            color: "#1a1c21",
            fontFamily: "system-ui, sans-serif",
          }}
        >
          <div style={{ maxWidth: "90vw", textAlign: "center" }}>
            <EuiText>Something went wrong</EuiText>
            <p style={{ marginBottom: 16 }}>
              If you can, please take a screenshot and share with{" "}
              <code>coverdrop@guardian.co.uk</code>
            </p>
            <pre
              style={{
                textAlign: "left",
                background: "#fff",
                border: "1px solid #d3dae6",
                borderRadius: 8,
                padding: 12,
                overflowX: "scroll",
                marginBottom: 16,
              }}
            >
              <strong>
                <code>{this.state.error.message}</code>
              </strong>
              <br />
              <br />
              {this.state.error.stack}
            </pre>
            <EuiButton iconType={"refresh"} onClick={this.handleReload}>
              Reload application
            </EuiButton>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
