import { EuiConfirmModal } from "@elastic/eui";
import {
  forceRotateJournalistIdPk,
  forceRotateMsgPk,
  forceRotateSentinelIdPk,
} from "../commands/admin";

export type KeyType = "msg" | "journalist-id" | "sentinel-id";

const keyTypeToPrettyName: Record<KeyType, string> = {
  msg: "messaging",
  "journalist-id": "journalist identity",
  "sentinel-id": "sentinel identity",
};

export const ForceRotateKeyModal = ({
  keyType,
  closeModal,
}: {
  keyType: KeyType;
  closeModal: () => void;
}): JSX.Element => {
  const title = `Are you sure you want to force a rotation?`;

  const prettyKeyType = keyTypeToPrettyName[keyType];

  const handleConfirm = async () => {
    try {
      // both force rotate commands involve network requests so don't await them
      if (keyType == "journalist-id") {
        forceRotateJournalistIdPk();
      } else if (keyType == "msg") {
        forceRotateMsgPk();
      } else if (keyType == "sentinel-id") {
        forceRotateSentinelIdPk();
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
