import {
  EuiCallOut,
  EuiButton,
  EuiModal,
  EuiModalHeaderTitle,
  EuiModalBody,
  EuiModalHeader,
  EuiSelect,
  EuiFormRow,
  EuiModalFooter,
} from "@elastic/eui";
import { usePublicInfoStore } from "../state/publicInfo";
import { useEffect, useState } from "react";
import { getBackupContacts, setBackupContacts } from "../commands/vaults";
import { JournalistIdentity } from "../model/bindings/JournalistIdentity";
import { Toast } from "@elastic/eui/src/components/toast/global_toast_list";
import { SetBackupContactReminderToastBody } from "./SetBackupContactReminderToastBody";

type ChooseBackupContactModalProps = {
  isOpen: boolean;
  journalistId: string;
  addCustomToast: (toast: Toast) => void;
  removeCustomToast: (id: string) => void;
  openModal: () => void;
  closeModal: () => void;
};

export const ChooseBackupContactModal = ({
  isOpen,
  journalistId,
  addCustomToast,
  removeCustomToast,
  openModal,
  closeModal,
}: ChooseBackupContactModalProps) => {
  const [selectedBackupContact, setSelectedBackupContact] =
    useState<JournalistIdentity | null>();
  const [
    shouldRequireSettingBackupContact,
    setShouldRequireSettingBackupContact,
  ] = useState<boolean>(false);

  const refreshIsSettingBackupContactRequired = () =>
    getBackupContacts().then((backupContacts: JournalistIdentity[]) => {
      setShouldRequireSettingBackupContact((prev) => {
        console.log("refreshIsSettingBackupContactRequired", {
          backupContacts,
          prev,
        });
        const needToSetContact = backupContacts.length === 0;
        if (needToSetContact && !prev) {
          const toastId = `backup-contact-${Date.now()}`;
          addCustomToast({
            id: toastId,
            title: "No backup contact set",
            color: "warning",
            iconType: "help",
            text: (
              <SetBackupContactReminderToastBody
                openBackupContactModal={openModal}
                remove={() => removeCustomToast(toastId)}
              />
            ),
            onClose: () => removeCustomToast(toastId),
          });
        }
        return needToSetContact;
      });

      if (backupContacts.length > 0) {
        setSelectedBackupContact(backupContacts[0]);
      }
    });

  useEffect(() => {
    refreshIsSettingBackupContactRequired();
    const timer = setInterval(
      refreshIsSettingBackupContactRequired,
      1000 * 60 /*every minute*/,
    );
    return () => clearInterval(timer);
  }, []);

  const submitHandler = async () => {
    if (!selectedBackupContact) {
      console.log("No backup contact selected");
      return;
    }
    console.log("Choose backup contact clicked");
    await setBackupContacts([selectedBackupContact]);

    setShouldRequireSettingBackupContact(false);
    closeModal();
  };

  const publicInfoStore = usePublicInfoStore();
  const publicInfo = publicInfoStore.getPublicInfo();
  if (!publicInfo) {
    console.log("Public info not available");
    return null;
  }

  // The backup candidates are all the other journalist profiles except this one.
  // TODO change this to the list of sentinel / vault identities
  const backupContactCandidates = publicInfo.journalist_profiles.filter(
    (p) => p.id !== journalistId,
  );

  const backupCandidateOptions = [{ value: "", text: "None" }].concat(
    backupContactCandidates
      .filter((candidate) => candidate.status === "VISIBLE")
      .sort((a, b) => (a.sort_name < b.sort_name ? -1 : 1))
      .map((candidate) => ({
        value: candidate.id,
        text: `${candidate.display_name} - ${candidate.description}`,
      })),
  );

  return (
    isOpen && (
      <EuiModal onClose={closeModal}>
        <EuiModalHeader>
          <EuiModalHeaderTitle>Choose Backup Contact</EuiModalHeaderTitle>
        </EuiModalHeader>
        <EuiModalBody>
          {shouldRequireSettingBackupContact && (
            <EuiFormRow>
              <EuiCallOut
                title="Choose a backup contact"
                color="warning"
                iconType="help"
              >
                <p>
                  Please choose a trusted contact who can help you recover your
                  vault if you lose access. This should be someone you trust
                  implicitly, as they will have the ability to assist in
                  recovering your data.
                </p>
              </EuiCallOut>
            </EuiFormRow>
          )}

          {backupContactCandidates.length === 0 ? (
            // this shouldn't ever happen
            <EuiCallOut
              title="No other journalist profiles available"
              color="warning"
              iconType="help"
            >
              <p>
                There are no other journalist profiles available to choose as a
                backup contact. Please contact your administrator for
                assistance.
              </p>
            </EuiCallOut>
          ) : (
            <EuiFormRow label="Select Backup Contact">
              <EuiSelect
                options={backupCandidateOptions}
                onChange={(e) => setSelectedBackupContact(e.target.value)}
                defaultValue={selectedBackupContact || ""}
              />
            </EuiFormRow>
          )}
        </EuiModalBody>
        <EuiModalFooter>
          <EuiButton
            onClick={submitHandler}
            isDisabled={!selectedBackupContact}
          >
            Submit
          </EuiButton>
        </EuiModalFooter>
      </EuiModal>
    )
  );
};
