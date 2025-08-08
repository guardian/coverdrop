import { JournalistStatus } from "../model/bindings/JournalistStatus.ts";
import { updateJournalistStatus } from "../commands/admin.ts";
import { EuiConfirmModal } from "@elastic/eui";
import { JournalistProfile } from "../model/bindings/JournalistProfile.ts";

export const ToggleJournalistStatusModal = ({
  journalistProfile,
  newStatus,
  closeModal,
  setJournalistStatus,
}: {
  journalistProfile: JournalistProfile;
  newStatus: JournalistStatus | null;
  closeModal: () => void;
  setJournalistStatus: (newStatus: JournalistStatus) => void;
}): JSX.Element | null => {
  if (!newStatus) {
    return null;
  }

  const title =
    newStatus === "HIDDEN_FROM_UI"
      ? "Hide me as a Secure Messaging recipient"
      : "Show me as a Secure Messaging recipient";
  const teamsOrJournalists = journalistProfile.is_desk
    ? "teams"
    : "journalists";
  const text =
    newStatus === "HIDDEN_FROM_UI"
      ? `This will set your Secure Messaging status to hidden, meaning the vault "${journalistProfile.display_name}" won't appear in the list of ${teamsOrJournalists} in the Guardian mobile app. New sources will not be able to contact you, but conversations that have already started will continue normally.`
      : `This will set your Secure Messaging status to visible, meaning the vault "${journalistProfile.display_name}" will appear in the list of ${teamsOrJournalists} in the Guardian mobile app, and sources will be able to contact you.`;
  const confirmButtonText =
    newStatus === "HIDDEN_FROM_UI" ? "Hide me" : "Show me";

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
      confirmButtonText={confirmButtonText}
      buttonColor="primary"
    >
      {text}
    </EuiConfirmModal>
  );
};
