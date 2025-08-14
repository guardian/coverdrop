import { EuiConfirmModal } from "@elastic/eui";
import { forceRotateIdPk, forceRotateMsgPk } from "../commands/admin";

export const ForceRotateKeyModal = ({
  keyType,
  closeModal,
}: {
  keyType: "msg" | "id";
  closeModal: () => void;
}): JSX.Element => {
  const title = `Are you sure you want to force a rotation?`;
  const prettyKeyType = keyType == "msg" ? "messaging" : "identity";

  const handleConfirm = async () => {
    try {
      // both force rotate commands involve network requests so don't await them
      if (keyType == "id") {
        forceRotateIdPk();
      } else if (keyType == "msg") {
        forceRotateMsgPk();
      }
      closeModal();
    } catch (e: unknown) {
      if (e instanceof Error) {
        console.error("Error handling forced key rotation:", e.message);
      } else {
        console.error("Error handling forced key rotation", e);
      }
    }
  };

  return (
    <EuiConfirmModal
      title={title}
      onCancel={closeModal}
      onConfirm={handleConfirm}
      cancelButtonText="Cancel"
      confirmButtonText="Rotate key"
      buttonColor="danger"
    >
      Are you sure you want to force a rotation of your {prettyKeyType} key?
    </EuiConfirmModal>
  );
};
