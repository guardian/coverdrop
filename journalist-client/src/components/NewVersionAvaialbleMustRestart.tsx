import { EuiButton, EuiSpacer, EuiText, EuiTitle } from "@elastic/eui";
import { softExitApp } from "../commands/admin.ts";

export const NewVersionAvailableMustRestart = () => (
  <div
    style={{
      textAlign: "center",
      display: "flex",
      flexDirection: "column",
      alignItems: "center",
      justifyContent: "center",
      height: "100vh",
    }}
  >
    <EuiTitle>
      <span>New Version Available</span>
    </EuiTitle>
    <EuiSpacer size="s" />
    <EuiText>
      There has been a new version available for over 24 hours.
      <br />
      You must restart the application to continue using it.
    </EuiText>
    <EuiSpacer size="s" />
    <EuiText>
      If you have problems updating then please contact{" "}
      <code>coverdrop@guardian.co.uk</code>
    </EuiText>

    <EuiSpacer size="m" />
    <EuiButton onClick={softExitApp}>Restart now</EuiButton>
  </div>
);
