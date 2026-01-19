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
  EuiFilePicker,
  EuiFlexGroup,
  EuiFlexItem,
} from "@elastic/eui";
import { ReactNode, useState } from "react";
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
  const [fileError, setFileError] = useState<ReactNode>("");

  const closeModalHandler = () => {
    setWrappedShare("");
    setUnwrappedShare("");
    setFileError("");
    closeModal();
  };

  const validateAndSetShareFromFile = async (file: File) => {
    if (!file.name.endsWith(".recovery-share.txt")) {
      setFileError(
        <span>
          Invalid file type. Please select a file ending with{" "}
          <code>.recovery-share.txt</code>
        </span>,
      );
      return;
    }

    try {
      const text = await file.text();
      setWrappedShare(text.trim());
      setFileError("");
    } catch (error) {
      setFileError("Failed to read file");
      console.error("Error reading file:", error);
    }
  };

  const handleFileSelection = async (files: FileList | null) => {
    if (!files || files.length === 0) {
      return;
    }

    if (files.length > 1) {
      setFileError("Please select only one file");
      return;
    }

    await validateAndSetShareFromFile(files[0]);
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
            Unwrap a backup secret share
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
                  into the text area below and click &quot;Submit&quot;, or use
                  the file picker to load the <code>.recovery-share.txt</code>{" "}
                  file the admin sent you.
                </p>
              </EuiCallOut>
            </EuiFormRow>
          )}
          {!unwrappedShare && (
            <EuiFormRow label="Wrapped secret share" fullWidth={true}>
              <EuiFlexGroup>
                <EuiFlexItem>
                  <EuiTextArea
                    placeholder="Paste wrapped backup secret share here"
                    onChange={(e) => setWrappedShare(e.target.value)}
                    value={wrappedShare}
                    fullWidth={true}
                    rows={5}
                  />
                </EuiFlexItem>
                <EuiFlexItem>
                  <EuiFilePicker
                    initialPromptText={
                      <span>
                        OR choose a <code>.recovery-share.txt</code> file
                      </span>
                    }
                    onChange={handleFileSelection}
                    accept=".recovery-share.txt"
                    aria-label="Select wrapped secret share file"
                    display="large"
                  />
                </EuiFlexItem>
              </EuiFlexGroup>
            </EuiFormRow>
          )}

          {!unwrappedShare && fileError && (
            <EuiFormRow fullWidth={true}>
              <EuiCallOut title="File Error" color="danger" iconType="alert">
                <p>{fileError}</p>
              </EuiCallOut>
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
