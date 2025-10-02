import { EuiButton, EuiFlexGroup, EuiFlexItem } from "@elastic/eui";

type SetBackupContactReminderToastBodyProps = {
  openBackupContactModal: () => void;
  remove: () => void;
};

export const SetBackupContactReminderToastBody = ({
  openBackupContactModal,
  remove,
}: SetBackupContactReminderToastBodyProps) => {
  return (
    <div>
      <p>
        You have not set a backup contact. Please choose a trusted contact to
        help you recover your vault if needed.
      </p>
      <EuiFlexGroup justifyContent="flexEnd" gutterSize="s">
        <EuiFlexItem grow={false}>
          <EuiButton
            size="s"
            onClick={() => {
              openBackupContactModal();
              remove();
            }}
          >
            Choose a backup contact
          </EuiButton>
        </EuiFlexItem>
      </EuiFlexGroup>
    </div>
  );
};
