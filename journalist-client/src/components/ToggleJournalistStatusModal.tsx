import { JournalistStatus } from "../model/bindings/JournalistStatus.ts";
import { updateJournalistStatus } from "../commands/admin.ts";
import { EuiConfirmModal } from "@elastic/eui";

export const ToggleJournalistStatusModal = ({
  newStatus,
  closeModal,
  setJournalistStatus,
}: {
  newStatus: JournalistStatus | null;
  closeModal: () => void;
  setJournalistStatus: (newStatus: JournalistStatus) => void;
}): JSX.Element | null => {
  if (!newStatus) {
    return null;
  }

  const title =
    newStatus === "HIDDEN_FROM_UI"
      ? "Set status to Hidden"
      : "Set status to Visible";
  const text =
    newStatus === "HIDDEN_FROM_UI"
      ? "Setting your status to hidden will hide your profile in the app. Sources will not be able to start new conversations with you, but conversations that have already started will continue normally."
      : "Setting your status to visible will allow app users to start new conversations with you.";

  const handleConfirm = async () => {
    try {
      await updateJournalistStatus(newStatus);

      setJournalistStatus(newStatus);

      closeModal();
    } catch (e: unknown) {
      if (e instanceof Error) {
        console.error("Error handling journalist status change:", e.message);
      } else {
        console.error("Error handling journalist status change", e);
      }
    }
  };

  return (
    <EuiConfirmModal
      title={title}
      onCancel={closeModal}
      onConfirm={handleConfirm}
      cancelButtonText="Cancel"
      confirmButtonText={title}
      buttonColor="primary"
    >
      {text}
    </EuiConfirmModal>
  );
};
