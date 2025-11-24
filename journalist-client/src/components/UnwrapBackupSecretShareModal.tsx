import {
  EuiCallOut,
  EuiButton,
  EuiModal,
  EuiModalHeaderTitle,
  EuiModalBody,
  EuiModalHeader,
  EuiFormRow,
  EuiModalFooter,
  EuiTextArea,
  EuiCodeBlock,
} from "@elastic/eui";
import { useState } from "react";
import { unwrapBackupSecretShare } from "../commands/backups";
import { sizes } from "../styles/sizes";

type RestoreBackupSecretShareModalProps = {
  isOpen: boolean;
  closeModal: () => void;
};

export const UnwrapBackupSecretShareModal = ({
  isOpen,
  closeModal,
}: RestoreBackupSecretShareModalProps) => {
  const [wrappedShare, setWrappedShare] = useState<string>("");
  const [unwrappedShare, setUnwrappedShare] = useState<string>("");

  const closeModalHandler = () => {
    setWrappedShare("");
    setUnwrappedShare("");
    closeModal();
  };
  const submitHandler = async () => {
    if (!wrappedShare) {
      console.log("No encrypted shares provided");
      return;
    }
    const unwrappedBackupSecretShare =
      await unwrapBackupSecretShare(wrappedShare);
    setUnwrappedShare(unwrappedBackupSecretShare);
  };

  return (
    isOpen && (
      <EuiModal
        onClose={closeModalHandler}
        style={sizes.restoreBackupSecretShareModal}
      >
        <EuiModalHeader>
          <EuiModalHeaderTitle>
            Unwrap a Backup Secret Share
          </EuiModalHeaderTitle>
        </EuiModalHeader>
        <EuiModalBody>
          {!unwrappedShare && (
            <EuiFormRow fullWidth={true}>
              <EuiCallOut
                title="Unwrap Backup Secret Share"
                color="danger"
                iconType="warning"
              >
                <p>
                  Use this form to unwrap an encrypted backup secret share
                  received from a Secure Messaging admin (a member of the I&R
                  team).
                </p>
                <p>
                  <b>
                    Before proceeding, verify the identity of the admin through
                    a trusted channel (e.g. phone call, in-person meeting).
                  </b>
                </p>
                <p>
                  Paste the encrypted backup secret share provided by the admin
                  into the text area below and click &quot;Submit&quot;.
                </p>
              </EuiCallOut>
            </EuiFormRow>
          )}
          {!unwrappedShare && (
            <EuiFormRow label="Wrapped secret share" fullWidth={true}>
              <EuiTextArea
                placeholder="Paste wrapped backup secret share here"
                onChange={(e) => setWrappedShare(e.target.value)}
                fullWidth={true}
              />
            </EuiFormRow>
          )}

          {unwrappedShare && (
            <EuiFormRow fullWidth={true}>
              <EuiCallOut
                title="Unwrapped Backup Secret Share"
                color="warning"
                iconType="help"
              >
                <p>
                  The unwrapped backup secret share is shown below. Copy it and
                  send it to the Secure Messaging admin who provided you with
                  the encrypted share <b>over a secure channel</b>.
                </p>
              </EuiCallOut>
            </EuiFormRow>
          )}

          {unwrappedShare && (
            <EuiCodeBlock
              language="markup"
              // makes it so long string doesn't overlap with copy button
              whiteSpace="pre"
              isCopyable={true}
            >
              {unwrappedShare}
            </EuiCodeBlock>
          )}
        </EuiModalBody>

        <EuiModalFooter>
          <EuiButton
            onClick={closeModalHandler}
            color={unwrappedShare ? "success" : "text"}
          >
            Close
          </EuiButton>
          {!unwrappedShare && (
            <EuiButton onClick={submitHandler} isDisabled={!wrappedShare}>
              Submit
            </EuiButton>
          )}
        </EuiModalFooter>
      </EuiModal>
    )
  );
};
