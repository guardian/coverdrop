import {
  MouseEvent,
  useState,
  useEffect,
  FormEvent,
  useRef,
  Fragment,
} from "react";

import {
  EuiButton,
  EuiFieldPassword,
  EuiFlexGroup,
  EuiForm,
  EuiFormRow,
  EuiLink,
  EuiPanel,
  EuiSelect,
  EuiSelectOption,
  EuiSpacer,
  EuiText,
  EuiTitle,
} from "@elastic/eui";
import {
  getColocatedPassword,
  unlockVault,
  getVaultState,
} from "../commands/vaults";
import { VaultState } from "../model/bindings/VaultState";
import { getProfiles } from "../commands/profiles";
import { open } from "@tauri-apps/plugin-dialog";
import { getLogs } from "../commands/admin";
import { LogsPanel } from "../components/LogsPanel";
import { SentinelLogEntry } from "../model/bindings/SentinelLogEntry";
import { VersionInfo } from "../components/VersionInfo.tsx";

type OpenVaultProps = {
  setVaultState: (s: VaultState) => void;
};

export const OpenVault = ({ setVaultState }: OpenVaultProps) => {
  const [profileOptions, setProfileOptions] = useState<EuiSelectOption[]>([]);

  const [busy, setBusy] = useState(false);
  const [profile, setProfile] = useState("");
  const [vaultPath, setVaultPath] = useState("");
  const [password, setPassword] = useState("");

  // logs
  const [isFlyoutVisible, setIsFlyoutVisible] = useState(false);
  const [logs, setLogs] = useState<SentinelLogEntry[]>([]);

  const getLogsClicked = () => {
    getLogs().then((logs) => {
      setLogs(logs);
      setIsFlyoutVisible(true);
    });
  };

  const passwordRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    getProfiles().then((p) => {
      const profileNames = Object.keys(p).map((k) => ({ text: k, value: k }));
      setProfileOptions(profileNames);
      if (profileNames.length > 0) {
        setProfile(profileNames[0].value);
      }
    });
  }, []);

  const pathParts = vaultPath.split(/[/\\]/);
  const vaultName = pathParts[pathParts.length - 1];

  let flyout = null;
  if (isFlyoutVisible) {
    flyout = (
      <LogsPanel
        logs={logs}
        setFlyoutVisible={setIsFlyoutVisible}
        refreshClicked={getLogsClicked}
      />
    );
  }
  return (
    <Fragment>
      <div
        style={{ paddingTop: "50px", display: "flex", justifyItems: "center" }}
      >
        <EuiPanel
          paddingSize="l"
          title="Unlock your vault"
          style={{ width: "400px", margin: "auto" }}
          grow={false}
        >
          <EuiTitle>
            <EuiText>Unlock your vault</EuiText>
          </EuiTitle>
          <EuiSpacer size="m" />
          <EuiForm
            component="form"
            onSubmit={async (e: FormEvent<HTMLFormElement>) => {
              e.preventDefault();
              setBusy(true);
              try {
                await unlockVault(profile, vaultPath, password);
                const state = await getVaultState();
                if (state !== null) {
                  setVaultState(state);
                }
              } finally {
                setBusy(false);
              }
            }}
          >
            <EuiFormRow label="Profile">
              <EuiSelect
                options={profileOptions}
                value={profile}
                onChange={(e) => {
                  setProfile(e.target.value);
                }}
              />
            </EuiFormRow>
            <EuiFormRow label="Select vault">
              <EuiFlexGroup dir="row" alignItems="center">
                <EuiButton
                  fullWidth
                  onClick={async () => {
                    const path = await open({
                      multiple: false,
                      filters: [{ name: "Vault", extensions: ["vault"] }],
                    });

                    if (Array.isArray(path)) {
                      console.error(
                        "Should not have got an array from file selection",
                      );
                      return;
                    }

                    if (path !== null) {
                      setVaultPath(path);

                      const colocatedPassword =
                        await getColocatedPassword(path);
                      if (colocatedPassword !== null) {
                        setPassword(colocatedPassword);
                      }

                      if (passwordRef.current) {
                        passwordRef.current.focus();
                      }
                    }
                  }}
                >
                  {vaultName ? vaultName : "No vault selected"}
                </EuiButton>
              </EuiFlexGroup>
            </EuiFormRow>
            <EuiFormRow label="Password">
              <EuiFieldPassword
                inputRef={passwordRef}
                type="dual"
                value={password}
                placeholder="Enter your password"
                style={{ textAlign: "center" }}
                onChange={(e) => {
                  setPassword(e.target.value);
                }}
                autoCapitalize="none"
                autoCorrect="off"
                spellCheck="false"
              />
            </EuiFormRow>
            <EuiFormRow>
              <div>
                <EuiButton
                  type="submit"
                  disabled={!vaultPath || password.length === 0}
                  isLoading={busy}
                  onClick={async (e: MouseEvent<HTMLButtonElement>) => {
                    e.preventDefault();
                    setBusy(true);
                    try {
                      await unlockVault(profile, vaultPath, password);
                      const state = await getVaultState();
                      if (state !== null) {
                        setVaultState(state);
                      }
                    } finally {
                      setBusy(false);
                    }
                  }}
                >
                  Open
                </EuiButton>
              </div>
            </EuiFormRow>
          </EuiForm>
        </EuiPanel>
      </div>
      <div
        style={{
          position: "fixed",
          bottom: 0,
          left: 0,
          padding: "5px",
        }}
      >
        <VersionInfo />
      </div>
      <EuiLink
        style={{
          position: "fixed",
          bottom: 0,
          right: 0,
          padding: "5px",
        }}
        onClick={getLogsClicked}
      >
        View logs
      </EuiLink>
      {flyout}
    </Fragment>
  );
};
