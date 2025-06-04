import { EuiFormRow, EuiRange, EuiConfirmModal } from "@elastic/eui";
import { burstCoverMessages } from "../commands/chats";
import { useState } from "react";
import { sizes } from "../styles/sizes";

export const BurstCoverMessageModal = (props: { closeModal: () => void }) => {
  const [busy, setBusy] = useState(false);
  const [count, setCount] = useState(1);

  return (
    <EuiConfirmModal
      style={{ width: sizes.coverMessageBurstModal.width }}
      title={<div>Send Cover Message Burst</div>}
      onCancel={props.closeModal}
      onConfirm={async () => {
        setBusy(true);
        await burstCoverMessages(count);
        setBusy(false);
        props.closeModal();
      }}
      isLoading={busy}
      cancelButtonText="Cancel"
      confirmButtonText="Send"
      buttonColor="primary"
    >
      <EuiFormRow>
        <EuiRange
          showInput
          value={count}
          min={1}
          max={1000}
          onChange={(v) => setCount(Number(v.currentTarget.value))}
        />
      </EuiFormRow>
    </EuiConfirmModal>
  );
};
