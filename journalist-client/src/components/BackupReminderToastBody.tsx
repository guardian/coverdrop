import { EuiButton, EuiFlexGroup, EuiFlexItem } from "@elastic/eui";
import { useEffect, useState } from "react";

type BackupReminderToastBodyProps = {
  setIsBackupModalOpen: (isOpen: boolean) => void;
  remove: () => void;
};

const gracePeriodInSeconds = 60;
const gracePeriodInMillis = gracePeriodInSeconds * 1000;

export const BackupReminderToastBody = ({
  setIsBackupModalOpen,
  remove,
}: BackupReminderToastBodyProps) => {
  const [remainingSeconds, setRemainingSeconds] =
    useState<number>(gracePeriodInSeconds);
  useEffect(() => {
    const epochMillisAtMount = Date.now();
    const timer = setInterval(() => {
      const newRemainingMillis =
        gracePeriodInMillis - Date.now() + epochMillisAtMount;
      if (newRemainingMillis < 0) {
        clearInterval(timer);
        setIsBackupModalOpen(true);
        remove();
      }
      setRemainingSeconds(Math.round(newRemainingMillis / 1000));
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  return (
    <div>
      <p>
        You must back up your vault and will be forced to do so in{" "}
        <strong>{remainingSeconds}s</strong>
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
