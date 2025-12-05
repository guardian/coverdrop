import { EuiFlyout, EuiFlyoutBody, EuiText, EuiTimeline } from "@elastic/eui";
import { BackupHistoryEntry } from "../model/bindings/BackupHistoryEntry";

type BackupHistoryFlyoutProps = {
  backupHistory: BackupHistoryEntry[];
  onClose: () => void;
};

export const BackupHistoryFlyout = ({
  backupHistory,
  onClose,
}: BackupHistoryFlyoutProps) => {
  return (
    <EuiFlyout onClose={onClose}>
      <EuiFlyoutBody>
        <EuiTimeline
          items={backupHistory.map((item) => ({
            icon: "dot",
            children: (
              <EuiText>
                {item.backupType} backup on{" "}
                {new Date(item.timestamp).toLocaleString()}
                {item.backupType == "AUTOMATED"
                  ? ` - contacts: ${item.recoveryContacts?.join(", ")}`
                  : ""}
              </EuiText>
            ),
          }))}
        />
      </EuiFlyoutBody>
    </EuiFlyout>
  );
};
