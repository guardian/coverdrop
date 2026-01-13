import { EuiButton, EuiFlexGroup, EuiFlexItem } from "@elastic/eui";
import { BackupAttemptFailureReason } from "../model/bindings/BackupAttemptFailureReason";

type BackupReminderToastBodyProps = {
  automaticBackupFailureReason: BackupAttemptFailureReason;
  setIsBackupModalOpen: (isOpen: boolean) => void;
  remove: () => void;
};

export const BackupReminderToastBody = ({
  automaticBackupFailureReason,
  setIsBackupModalOpen,
  remove,
}: BackupReminderToastBodyProps) => {
  const failureReasonMessageMap: Record<BackupAttemptFailureReason, string> = {
    VAULT_TOO_LARGE:
      "the vault size exceeds maximum backup size. Please contact the I&R team and perform a manual backup.",
    INSUFFICIENT_RECOVERY_CONTACTS_SELECTED:
      "not enough recovery contacts are selected. Please select more recovery contacts and perform a manual backup.",
    INSUFFICIENT_RECOVERY_CONTACTS_WITH_VALID_KEYS:
      "not enough recovery contacts have valid keys. Please update your recovery contacts and perform a manual backup.",
    S3: "an S3 storage error occurred. Please perform a manual backup.",
    UNKNOWN: "an unknown error occurred. Please perform a manual backup.",
  };
  return (
    <div>
      <p>
        Automated vault backup failed because{" "}
        {failureReasonMessageMap[automaticBackupFailureReason]}
      </p>
      <EuiFlexGroup justifyContent="flexEnd" gutterSize="s">
        <EuiFlexItem grow={false}>
          <EuiButton
            size="s"
            onClick={() => {
              setIsBackupModalOpen(true);
              remove();
            }}
          >
            Back up now
          </EuiButton>
        </EuiFlexItem>
      </EuiFlexGroup>
    </div>
  );
};
