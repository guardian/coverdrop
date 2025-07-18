import { Fragment, useState } from "react";
import {
  EuiButtonIcon,
  EuiContextMenuItem,
  EuiContextMenuPanel,
  EuiPopover,
} from "@elastic/eui";
import {
  forceRotateIdPk,
  forceRotateMsgPk,
  getLogs,
  getPublicInfo,
  getVaultKeys,
} from "../commands/admin";
import { LogsPanel } from "./LogsPanel";
import { PublicInfoPanel } from "./PublicInfoPanel";
import { BurstCoverMessageModal } from "./BurstMessageModal";
import { VaultKeysPanel } from "./VaultKeysPanel";
import { TrustedKeyDigestsModal } from "./TrustedKeyDigestsModal";
import { AddTrustAnchorModal } from "./AddTrustAnchorModal";
import { SentinelLogEntry } from "../model/bindings/SentinelLogEntry";

type FlyoverContent =
  | {
      type: "vault-keys";
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      json: any;
    }
  | {
      type: "logs";
      logs: SentinelLogEntry[];
    }
  | {
      type: "public-info";
      json: string;
    };

export const SettingsPopover = () => {
  const [isFlyoutVisible, setIsFlyoutVisible] = useState(false);
  const [flyoutContent, setFlyoutContent] = useState<FlyoverContent | null>(
    null,
  );
  const [burstCoverMessagesModalVisible, setBurstCoverMessagesModalVisible] =
    useState(false);

  const [trustedKeyDigestModalVisible, setTrustedKeyDigestModalVisible] =
    useState(false);

  const [addTrustAnchorModalVisible, setAddTrustAnchorModalVisible] =
    useState(false);

  const [isPopoverOpen, setIsPopoverOpen] = useState(false);

  const forceIdRotationClicked = () => {
    forceRotateIdPk();
    setIsPopoverOpen(false);
  };

  const forceMsgRotationClicked = () => {
    forceRotateMsgPk();
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
        json: i,
      });
      setIsFlyoutVisible(true);
    });
    setIsPopoverOpen(false);
  };

  const getLogsClicked = () => {
    getLogs().then((logs) => {
      setFlyoutContent({
        type: "logs",
        logs,
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

  let flyout = null;

  if (isFlyoutVisible) {
    if (flyoutContent?.type === "logs") {
      flyout = (
        <LogsPanel
          logs={flyoutContent.logs}
          setFlyoutVisible={setIsFlyoutVisible}
          refreshClicked={getLogsClicked}
        />
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
  }

  let burstCoverMessagesModal = null;
  if (burstCoverMessagesModalVisible) {
    burstCoverMessagesModal = (
      <BurstCoverMessageModal
        closeModal={() => {
          setBurstCoverMessagesModalVisible(false);
        }}
      />
    );
  }

  let trustedKeyDigestModal = null;
  if (trustedKeyDigestModalVisible) {
    trustedKeyDigestModal = (
      <TrustedKeyDigestsModal
        closeModal={() => {
          setTrustedKeyDigestModalVisible(false);
        }}
      />
    );
  }

  let addTrustAnchorModal = null;
  if (addTrustAnchorModalVisible) {
    addTrustAnchorModal = (
      <AddTrustAnchorModal
        closeModal={() => {
          setAddTrustAnchorModalVisible(false);
        }}
      />
    );
  }

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
           <EuiContextMenuItem icon="list" onClick={getLogsClicked}>
            View application logs
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="lock" onClick={getVaultKeysClicked}>
            View vault keys
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="eye" onClick={getPublicInfoClicked}>
            View all public profiles and keys
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="index" onClick={trustedKeyDigestClicked}>
            View trust anchor digests
          </EuiContextMenuItem>
          <EuiContextMenuItem>
            <strong>Danger Zone</strong>
          </EuiContextMenuItem>
          <EuiContextMenuItem
            color="warning"
            icon="sun"
            onClick={burstCoverMessagesClicked}
          >
            Send cover message burst
          </EuiContextMenuItem>
          <EuiContextMenuItem
            color="warning"
            icon="timeRefresh"
            onClick={forceIdRotationClicked}
          >
            Force identity key rotation
          </EuiContextMenuItem>
          <EuiContextMenuItem
            color="warning"
            icon="timeRefresh"
            onClick={forceMsgRotationClicked}
          >
            Force messaging key rotation
          </EuiContextMenuItem>
          <EuiContextMenuItem
            color="warning"
            icon="key"
            onClick={addTrustAnchorClicked}
          >
            Add trust anchor
          </EuiContextMenuItem>
        </EuiContextMenuPanel>
      </EuiPopover>
      {flyout}
      {burstCoverMessagesModal}
      {trustedKeyDigestModal}
      {addTrustAnchorModal}
    </Fragment>
  );
};
