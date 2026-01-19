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
  performBackup,
} from "../commands/backups.ts";
import { BackupChecks } from "../model/bindings/BackupChecks.ts";
import { Toast } from "@elastic/eui/src/components/toast/global_toast_list";
import { BackupReminderToastBody } from "./BackupReminderToastBody.tsx";
import { ask } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { BackupAttemptFailureReason } from "../model/bindings/BackupAttemptFailureReason.ts";

type BackupModalProps = {
  isOpen: boolean;
  vaultPath: string;
  setIsBackupModalOpen: (isOpen: boolean) => void;
  addCustomToast: (toast: Toast) => void;
  removeCustomToast: (toastId: string) => void;
};

export const ManualBackupModal = ({
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
        title: "Manual backup required",
        kind: "warning",
        okLabel: "Return",
        cancelLabel: "Continue without backing up",
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

  // Listen for event from backend indicating a manual backup is required
  // and show the toast if it is.
  useEffect(() => {
    const listener = listen<BackupAttemptFailureReason | null>(
      "manual_backup_required",
      (event) => {
        setIsBackupRequired((prev) => {
          if (event.payload !== null && !prev) {
            const toastId = `backup-preferred-${Date.now()}`;
            addCustomToast({
              id: toastId,
              title: "Manual backup required",
              color: "warning",
              iconType: "warning",
              onClose: async () => {
                if (await confirmIgnoringBackupRequired()) {
                  removeCustomToast(toastId);
                }
              },
              text: (
                <BackupReminderToastBody
                  automaticBackupFailureReason={event.payload}
                  setIsBackupModalOpen={setIsBackupModalOpen}
                  remove={() => removeCustomToast(toastId)}
                />
              ),
            });
          }
          return event.payload !== null;
        });
      },
    );
    return () => {
      listener.then((unlisten) => unlisten());
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
      "Backup complete. Would you like to 'eject' the backup volume now, so you can then unplug it?",
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
          "Successfully ejected the backup volume. You can now safely remove the backup usb stick from your computer.",
        );
      } else {
        alert(
          "Failed to eject the backup volume. Perhaps it has already been ejected? If not, you'll need to do so manually.",
        );
      }
    }
    setIsBackingUp(false);
    setIsBackupRequired(false);
  }, []);

  return !isOpen ? null : (
    <EuiModal onClose={closeModal}>
      <EuiModalHeader>
        <EuiModalHeaderTitle>Perform USB vault back up</EuiModalHeaderTitle>
      </EuiModalHeader>
      <EuiModalBody>
        <b>Note:</b> Your vault is automatically and securely backed up to the
        cloud whenever important cryptographic keys are replenished. If
        automated backups fail, to ensure your data is safe you must perform a
        manual backup to a specially prepared SentinelBackup USB stick.
        <br />
        {isBackupRequired && (
          <EuiCallOut
            iconType="warning"
            color="danger"
            title="Important: please back up your vault before continuing to use Sentinel"
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
                  : "SentinelBackup volume is not encrypted. Please contact the development team."}
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
