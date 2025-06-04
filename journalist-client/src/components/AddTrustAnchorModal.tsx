import { EuiConfirmModal, EuiCallOut, EuiButton } from "@elastic/eui";
import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { sizes } from "../styles/sizes";
import { addTrustAnchor } from "../commands/vaults";

export const AddTrustAnchorModal = (props: { closeModal: () => void }) => {
  const [busy, setBusy] = useState(false);
  const [path, setPath] = useState("");

  const pathParts = path.split(/[/\\]/);
  const filename = pathParts[pathParts.length - 1];

  return (
    <EuiConfirmModal
      style={{ width: sizes.addTrustAnchorModal.width }}
      title={<div>Add Trust Anchor</div>}
      onCancel={props.closeModal}
      onConfirm={async () => {
        setBusy(true);
        addTrustAnchor(path);
        setBusy(false);
        props.closeModal();
      }}
      isLoading={busy}
      cancelButtonText="Cancel"
      confirmButtonText="Add"
      buttonColor="primary"
    >
      <EuiCallOut
        title="Proceed with caution!"
        color="warning"
        iconType="warning"
      >
        <p>
          Only ever add a new trust anchor at the direct and verifiable
          instruction of an administrator.
        </p>
        <p>
          If you are not absolutely confident that the person who instructed you
          to add this new anchor is who they sey they are then do{" "}
          <strong>NOT</strong> proceed.
        </p>
        <EuiButton
          onClick={async () => {
            const path = await open({
              multiple: false,
              filters: [{ name: "Public Key", extensions: ["json"] }],
            });

            if (Array.isArray(path)) {
              console.error("Should not have got an array from file selection");
              return;
            }

            if (path !== null) {
              setPath(path);
            }
          }}
        >
          {path ? filename : "Select organization key"}
        </EuiButton>
      </EuiCallOut>
    </EuiConfirmModal>
  );
};
