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
      title="Send cover message burst"
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
      <div>
        <p>
          Send journalist-to-user cover traffic to the CoverNode.
          <br />
          This is a testing feature intended for developer use only.
        </p>
        <EuiFormRow label="Number of messages">
          <EuiRange
            showInput
            value={count}
            min={1}
            max={1000}
            onChange={(v) => setCount(Number(v.currentTarget.value))}
          />
        </EuiFormRow>
      </div>
    </EuiConfirmModal>
  );
};
