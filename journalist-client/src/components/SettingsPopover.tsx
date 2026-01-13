import { Fragment, useState } from "react";
import {
  EuiButton,
  EuiButtonIcon,
  EuiContextMenuItem,
  EuiContextMenuPanel,
  EuiFlyout,
  EuiLink,
  EuiPopover,
  EuiSpacer,
} from "@elastic/eui";
import {
  getPublicInfo,
  getVaultKeys,
  launchNewSentinelInstance,
} from "../commands/admin";
import { Logs } from "./Logs.tsx";
import { PublicInfoPanel } from "./PublicInfoPanel";
import { BurstCoverMessageModal } from "./BurstMessageModal";
import { VaultKeysPanel } from "./VaultKeysPanel";
import { TrustedKeyDigestsModal } from "./TrustedKeyDigestsModal";
import { AddTrustAnchorModal } from "./AddTrustAnchorModal";
import { JournalistStatus } from "../model/bindings/JournalistStatus";
import { ForceRotateKeyModal } from "./ForceRotateKeyModal";
import { ChooseBackupContactModal } from "./ChooseBackupContactModal";
import { Toast } from "@elastic/eui/src/components/toast/global_toast_list";
import { VersionInfo } from "./VersionInfo.tsx";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { UnwrapBackupSecretShareModal } from "./UnwrapBackupSecretShareModal";
import { BackupHistoryFlyout } from "./BackupHistory.tsx";
import { getBackupHistory } from "../commands/backups.ts";
import { BackupHistoryEntry } from "../model/bindings/BackupHistoryEntry.ts";

type FlyoutContent =
  | {
      type: "vault-keys";
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      json: any;
    }
  | {
      type: "logs";
    }
  | {
      type: "public-info";
      json: string;
    }
  | {
      type: "backup-history";
      backupHistory: BackupHistoryEntry[];
    };

interface SettingsPopoverProps {
  journalistId: string;
  journalistStatus?: JournalistStatus;
  devMode: boolean;
  setMaybeJournalistStatusForModal: (
    newStatus: JournalistStatus | null,
  ) => void;
  addCustomToast: (toast: Toast) => void;
  removeCustomToast: (id: string) => void;
  openBackupModal: () => void;
}

export const SettingsPopover = ({
  journalistId,
  journalistStatus,
  devMode,
  setMaybeJournalistStatusForModal,
  addCustomToast,
  removeCustomToast,
  openBackupModal,
}: SettingsPopoverProps) => {
  const [isFlyoutVisible, setIsFlyoutVisible] = useState(false);
  const [flyoutContent, setFlyoutContent] = useState<FlyoutContent | null>(
    null,
  );
  const [burstCoverMessagesModalVisible, setBurstCoverMessagesModalVisible] =
    useState(false);

  const [trustedKeyDigestModalVisible, setTrustedKeyDigestModalVisible] =
    useState(false);

  const [addTrustAnchorModalVisible, setAddTrustAnchorModalVisible] =
    useState(false);

  const [forceRotateKeyType, setForceRotateKeyType] = useState<
    "msg" | "id" | null
  >(null);

  const [chooseBackupContactModalVisible, setChooseBackupContactModalVisible] =
    useState(false);

  const [
    restoreBackupSecretShareModalVisible,
    setRestoreBackupSecretShareModalVisible,
  ] = useState(false);

  const chooseBackupContactClicked = () => {
    setChooseBackupContactModalVisible(true);
    setIsPopoverOpen(false);
  };
  const [isPopoverOpen, setIsPopoverOpen] = useState(false);

  const setStatusClicked = (newStatus: JournalistStatus) => {
    setMaybeJournalistStatusForModal(newStatus);
    setIsPopoverOpen(false);
  };

  const getVaultKeysClicked = () => {
    getVaultKeys().then((i) => {
      setFlyoutContent({
        type: "vault-keys",
        json: i,
      });
      setIsFlyoutVisible(true);
    });
    setIsPopoverOpen(false);
  };

  const getPublicInfoClicked = () => {
    getPublicInfo().then((i) => {
      setFlyoutContent({
        type: "public-info",
        json: JSON.stringify(i, null, 3),
      });
      setIsFlyoutVisible(true);
    });
    setIsPopoverOpen(false);
  };

  const getLogsClicked = () => {
    setFlyoutContent({
      type: "logs",
    });
    setIsFlyoutVisible(true);
    setIsPopoverOpen(false);
  };

  const backupHistoryClicked = () => {
    getBackupHistory().then((backupHistory) => {
      setFlyoutContent({
        type: "backup-history",
        backupHistory,
      });
      setIsFlyoutVisible(true);
      setIsPopoverOpen(false);
    });
  };

  const addTrustAnchorClicked = () => {
    setAddTrustAnchorModalVisible(true);
    setIsPopoverOpen(false);
  };

  const burstCoverMessagesClicked = () => {
    setBurstCoverMessagesModalVisible(true);
    setIsPopoverOpen(false);
  };

  const trustedKeyDigestClicked = () => {
    setTrustedKeyDigestModalVisible(true);
    setIsPopoverOpen(false);
  };

  const backUpVaultClicked = () => {
    openBackupModal();
    setIsPopoverOpen(false);
  };

  const openLogsInNewWindow = async () => {
    setIsFlyoutVisible(false);
    const label = "logs";
    const maybeExistingWindow = await WebviewWindow.getByLabel(label);
    if (maybeExistingWindow) {
      await maybeExistingWindow.show();
    } else {
      const webview = new WebviewWindow(label, {
        url: "logs.html",
        title: "Logs (Sentinel)", // TODO get the app name plus 'Logs'
      });
      await webview.once("tauri://error", (e) => {
        console.error("Error creating webview window:", e);
      });
    }
  };

  const restoreBackupSecretShareClicked = () => {
    setRestoreBackupSecretShareModalVisible(true);
    setIsPopoverOpen(false);
  };

  let flyout = null;

  if (isFlyoutVisible) {
    if (flyoutContent?.type === "logs") {
      flyout = (
        <EuiFlyout size="l" onClose={() => setIsFlyoutVisible(false)}>
          <EuiButton onClick={openLogsInNewWindow}>
            Open in new window
          </EuiButton>
          <Logs />
        </EuiFlyout>
      );
    }

    if (flyoutContent?.type === "public-info") {
      flyout = (
        <PublicInfoPanel
          json={flyoutContent.json}
          setFlyoutVisible={setIsFlyoutVisible}
          refreshClicked={getPublicInfoClicked}
        />
      );
    }

    if (flyoutContent?.type === "vault-keys") {
      flyout = (
        <VaultKeysPanel
          json={flyoutContent.json}
          setFlyoutVisible={setIsFlyoutVisible}
          refreshClicked={getVaultKeysClicked}
        />
      );
    }

    if (flyoutContent?.type === "backup-history") {
      flyout = (
        <BackupHistoryFlyout
          backupHistory={flyoutContent.backupHistory}
          onClose={() => setIsFlyoutVisible(false)}
        />
      );
    }
  }

  const burstCoverMessagesModal = burstCoverMessagesModalVisible && (
    <BurstCoverMessageModal
      closeModal={() => {
        setBurstCoverMessagesModalVisible(false);
      }}
    />
  );

  const trustedKeyDigestModal = trustedKeyDigestModalVisible && (
    <TrustedKeyDigestsModal
      closeModal={() => {
        setTrustedKeyDigestModalVisible(false);
      }}
    />
  );

  const addTrustAnchorModal = addTrustAnchorModalVisible && (
    <AddTrustAnchorModal
      closeModal={() => {
        setAddTrustAnchorModalVisible(false);
      }}
    />
  );

  const forceRotateKeyModal = forceRotateKeyType && (
    <ForceRotateKeyModal
      closeModal={() => {
        setForceRotateKeyType(null);
      }}
      keyType={forceRotateKeyType}
    />
  );

  const chooseBackupContactModal = (
    <ChooseBackupContactModal
      isOpen={chooseBackupContactModalVisible}
      journalistId={journalistId}
      addCustomToast={addCustomToast}
      removeCustomToast={removeCustomToast}
      closeModal={() => {
        setChooseBackupContactModalVisible(false);
      }}
      openModal={() => setChooseBackupContactModalVisible(true)}
    />
  );

  const restoreBackupSecretShareModal = (
    <UnwrapBackupSecretShareModal
      isOpen={restoreBackupSecretShareModalVisible}
      closeModal={() => {
        setRestoreBackupSecretShareModalVisible(false);
      }}
    />
  );

  return (
    <Fragment>
      <EuiPopover
        panelPaddingSize="none"
        button={
          <EuiButtonIcon
            iconType="menu"
            onClick={() => setIsPopoverOpen(!isPopoverOpen)}
          ></EuiButtonIcon>
        }
        isOpen={isPopoverOpen}
        closePopover={() => setIsPopoverOpen(false)}
      >
        <EuiContextMenuPanel>
          <EuiContextMenuItem>
            <strong>Helpers</strong>
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="documentEdit">
            <EuiLink
              href="https://docs.google.com/document/d/1QvVbPchfN5Hqf9TuNQ9QPHFeRJXiXN9P4H-ZBmDojsE"
              target="_blank"
              rel="noopener noreferrer"
              color="text"
            >
              Standard replies
            </EuiLink>
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="list" onClick={getLogsClicked}>
            View application logs
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="inspect" onClick={getPublicInfoClicked}>
            View all public profiles and keys
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="index" onClick={trustedKeyDigestClicked}>
            View trust anchor digests
          </EuiContextMenuItem>
          {
            <EuiContextMenuItem
              icon="accessibility"
              onClick={chooseBackupContactClicked}
            >
              Choose backup contact
            </EuiContextMenuItem>
          }
          <EuiContextMenuItem icon="save" onClick={backUpVaultClicked}>
            Perform USB vault back up
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="index" onClick={backupHistoryClicked}>
            Backup history
          </EuiContextMenuItem>
          {journalistStatus == "VISIBLE" && (
            <EuiContextMenuItem
              icon="eyeClosed"
              onClick={() => setStatusClicked("HIDDEN_FROM_UI")}
            >
              Hide me as a Secure Messaging recipient
            </EuiContextMenuItem>
          )}
          {journalistStatus == "HIDDEN_FROM_UI" && (
            <EuiContextMenuItem
              icon="eye"
              onClick={() => setStatusClicked("VISIBLE")}
            >
              Show me as a Secure Messaging recipient
            </EuiContextMenuItem>
          )}
          {journalistStatus === undefined && (
            <EuiContextMenuItem icon="eye" disabled={true}>
              Status pending
            </EuiContextMenuItem>
          )}
          <EuiContextMenuItem>
            <strong>Danger Zone</strong>
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="key" onClick={getVaultKeysClicked}>
            View vault keys
          </EuiContextMenuItem>
          {devMode && (
            <EuiContextMenuItem icon="sun" onClick={burstCoverMessagesClicked}>
              Send cover message burst
            </EuiContextMenuItem>
          )}
          <EuiContextMenuItem
            icon="timeRefresh"
            onClick={() => {
              setForceRotateKeyType("id");
              setIsPopoverOpen(false);
            }}
          >
            Force identity key rotation
          </EuiContextMenuItem>
          <EuiContextMenuItem
            icon="timeRefresh"
            onClick={() => {
              setForceRotateKeyType("msg");
              setIsPopoverOpen(false);
            }}
          >
            Force messaging key rotation
          </EuiContextMenuItem>
          {devMode && (
            <EuiContextMenuItem icon="link" onClick={addTrustAnchorClicked}>
              Add trust anchor
            </EuiContextMenuItem>
          )}
          <EuiContextMenuItem
            icon="popout"
            onClick={async () => {
              setIsPopoverOpen(false);
              await launchNewSentinelInstance();
            }}
          >
            Open another Sentinel instance
          </EuiContextMenuItem>

          {
            <EuiContextMenuItem
              icon="lockOpen"
              onClick={restoreBackupSecretShareClicked}
            >
              Restore backup secret share
            </EuiContextMenuItem>
          }
          <EuiContextMenuItem size="s">
            <EuiSpacer size="s" />
            <VersionInfo />
          </EuiContextMenuItem>
        </EuiContextMenuPanel>
      </EuiPopover>
      {flyout}
      {burstCoverMessagesModal}
      {trustedKeyDigestModal}
      {addTrustAnchorModal}
      {forceRotateKeyModal}
      {chooseBackupContactModal}
      {restoreBackupSecretShareModal}
    </Fragment>
  );
};
