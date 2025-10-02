import {
  EuiButton,
  EuiButtonEmpty,
  EuiCallOut,
  EuiFlexGroup,
  EuiIcon,
  EuiModal,
  EuiModalBody,
  EuiModalFooter,
  EuiModalHeader,
  EuiModalHeaderTitle,
} from "@elastic/eui";
import { useCallback, useEffect, useState } from "react";
import {
  ejectBackupVolume,
  getBackupChecks,
  getShouldRequireBackup,
  performBackup,
} from "../commands/vaults.ts";
import { BackupChecks } from "../model/bindings/BackupChecks.ts";
import { Toast } from "@elastic/eui/src/components/toast/global_toast_list";
import { BackupReminderToastBody } from "./BackupReminderToastBody.tsx";
import { ask } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

type BackupModalProps = {
  isOpen: boolean;
  vaultPath: string;
  setIsBackupModalOpen: (isOpen: boolean) => void;
  addCustomToast: (toast: Toast) => void;
  removeCustomToast: (toastId: string) => void;
};

export const BackupModal = ({
  isOpen,
  vaultPath,
  addCustomToast,
  removeCustomToast,
  setIsBackupModalOpen,
}: BackupModalProps) => {
  const [isBackupRequired, setIsBackupRequired] = useState<boolean>();
  const [isBackingUp, setIsBackingUp] = useState(false);

  const confirmIgnoringBackupRequired = useCallback(async () => {
    const shouldReturn = await ask(
      "It's crucial to back up your vault frequently. If you really cannot back up now, you can choose to continue without backing up.",
      {
        title: "Back up Required",
        kind: "warning",
        okLabel: "Return",
        cancelLabel: "Continue WITHOUT backing up",
      },
    );
    if (!shouldReturn) {
      setIsBackupModalOpen(false);
    }
    return !shouldReturn;
  }, []);

  const closeModal = useCallback(async () => {
    if (isBackupRequired) {
      await confirmIgnoringBackupRequired();
    } else if (!isBackingUp) {
      setIsBackupModalOpen(false);
    }
  }, [isBackupRequired]);

  const refreshIsBackupRequired = () =>
    getShouldRequireBackup().then((shouldRequireBackup) =>
      setIsBackupRequired((prev) => {
        if (shouldRequireBackup && !prev) {
          const toastId = `backup-preferred-${Date.now()}`;
          addCustomToast({
            id: toastId,
            title: "Back up Required",
            color: "warning",
            iconType: "warning",
            onClose: async () => {
              if (await confirmIgnoringBackupRequired()) {
                removeCustomToast(toastId);
              }
            },
            text: (
              <BackupReminderToastBody
                setIsBackupModalOpen={setIsBackupModalOpen}
                remove={() => removeCustomToast(toastId)}
              />
            ),
          });
        }
        return shouldRequireBackup;
      }),
    );

  useEffect(() => {
    refreshIsBackupRequired();
    const unlistenFnPromise = listen(
      "journalist_keys_rotated",
      refreshIsBackupRequired,
    );
    return () => {
      unlistenFnPromise.then((unlisten) => unlisten());
    };
  }, []);

  const [backupChecks, setBackupChecks] = useState<BackupChecks | null>(null);

  const runBackupChecks = useCallback(
    () => isOpen && getBackupChecks().then(setBackupChecks),
    [isOpen],
  );

  useEffect(() => {
    if (isOpen) {
      const interval = setInterval(runBackupChecks, 1000);
      return () => clearInterval(interval);
    }
  }, [isOpen]);

  useEffect(() => {
    if (isOpen) {
      runBackupChecks();
    }
  }, [isOpen]);

  const backup = useCallback(async () => {
    setIsBackingUp(true);
    await performBackup();
    setIsBackupModalOpen(false);
    const shouldEjectBackupVolume = await ask(
      "Back up complete! Would you like to 'eject' the backup volume now, so you can then remove it.",
      {
        title: "Back up complete",
        kind: "info",
        okLabel: "Eject (recommended)",
        cancelLabel: "Don't eject",
      },
    );
    if (shouldEjectBackupVolume) {
      if (await ejectBackupVolume()) {
        alert(
          "Successfully ejected the backup volume. You can now safely remove the backup usb stick from your machine.",
        );
      } else {
        alert(
          "Failed to eject the backup volume. Perhaps it has already been ejected, otherwise you'll need to do this manually.",
        );
      }
    }
    setIsBackingUp(false);
    await refreshIsBackupRequired();
  }, []);

  return !isOpen ? null : (
    <EuiModal onClose={closeModal}>
      <EuiModalHeader>
        <EuiModalHeaderTitle>Back up vault</EuiModalHeaderTitle>
      </EuiModalHeader>
      <EuiModalBody>
        {isBackupRequired && (
          <EuiCallOut
            iconType="warning"
            color="danger"
            title="IMPORTANT: you MUST back up your vault before you can continue to use Sentinel."
          />
        )}
        <br />
        <p>
          Current vault path: <code>{vaultPath}</code>
        </p>
        <br />
        {!backupChecks ? (
          "Backup checks running..."
        ) : (
          <div>
            <EuiFlexGroup alignItems="center" gutterSize="xs">
              <EuiIcon
                type={backupChecks.isBackupVolumeMounted ? "check" : "cross"}
                color={
                  backupChecks.isBackupVolumeMounted ? "success" : "danger"
                }
              />
              {backupChecks.isBackupVolumeMounted
                ? "SentinelBackup volume detected"
                : "Please insert the SentinelBackup USB stick"}
            </EuiFlexGroup>
            <br />
            {backupChecks.isBackupVolumeMounted && (
              <EuiFlexGroup alignItems="center" gutterSize="xs">
                <EuiIcon
                  type={backupChecks.isEncrypted ? "check" : "cross"}
                  color={backupChecks.isEncrypted ? "success" : "danger"}
                />
                {backupChecks.isEncrypted
                  ? "SentinelBackup volume is encrypted"
                  : "SentinelBackup volume is not encrypted. Please contact the I&R team."}
              </EuiFlexGroup>
            )}
            {backupChecks.maybeExistingBackups &&
              backupChecks.maybeExistingBackups.length > 0 && (
                <div>
                  <br />
                  <strong>Existing Backups:</strong>
                  <ul style={{ maxHeight: "100px", overflowY: "auto" }}>
                    {backupChecks.maybeExistingBackups.map((backup) => (
                      <li key={backup}>{backup}</li>
                    ))}
                  </ul>
                </div>
              )}
          </div>
        )}
      </EuiModalBody>
      <EuiModalFooter>
        <EuiButtonEmpty onClick={closeModal} disabled={isBackingUp}>
          Cancel
        </EuiButtonEmpty>
        <EuiButton
          type="submit"
          fill
          isLoading={isBackingUp}
          disabled={
            isBackingUp ||
            !backupChecks?.isBackupVolumeMounted ||
            !backupChecks?.isEncrypted
          }
          onClick={backup}
        >
          Back up
        </EuiButton>
      </EuiModalFooter>
    </EuiModal>
  );
};
