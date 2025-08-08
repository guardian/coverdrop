import { Fragment, useState } from "react";
import {
  EuiButtonIcon,
  EuiContextMenuItem,
  EuiContextMenuPanel,
  EuiLink,
  EuiPopover,
  EuiSpacer,
  EuiText,
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
import { JournalistStatus } from "../model/bindings/JournalistStatus";

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

export const SettingsPopover = ({
  journalistStatus,
  setMaybeJournalistStatusForModal,
}: {
  journalistStatus?: JournalistStatus;
  setMaybeJournalistStatusForModal: (
    newStatus: JournalistStatus | null,
  ) => void;
}) => {
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

  const setStatusClicked = (newStatus: JournalistStatus) => {
    setMaybeJournalistStatusForModal(newStatus);
    setIsPopoverOpen(false);
  };

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
        json: JSON.stringify(i, null, 3),
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

  const maybeRepo = import.meta.env.VITE_GITHUB_REPO;
  const maybeGithubRepoName = maybeRepo.startsWith("git@")
    ? maybeRepo.substring(maybeRepo.indexOf(":") + 1, maybeRepo.length - 4) // local repo ssh
    : maybeRepo?.startsWith("https://github.com/")
      ? maybeRepo.substring(
          // https (locally or in GHA)
          19,
          maybeRepo.endsWith(".git") ? maybeRepo.length - 4 : maybeRepo.length,
        )
      : maybeRepo;

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
          <EuiContextMenuItem icon="key" onClick={getVaultKeysClicked}>
            View vault keys
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="inspect" onClick={getPublicInfoClicked}>
            View all public profiles and keys
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="index" onClick={trustedKeyDigestClicked}>
            View trust anchor digests
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
          <EuiContextMenuItem icon="sun" onClick={burstCoverMessagesClicked}>
            Send cover message burst
          </EuiContextMenuItem>
          <EuiContextMenuItem
            icon="timeRefresh"
            onClick={forceIdRotationClicked}
          >
            Force identity key rotation
          </EuiContextMenuItem>
          <EuiContextMenuItem
            icon="timeRefresh"
            onClick={forceMsgRotationClicked}
          >
            Force messaging key rotation
          </EuiContextMenuItem>
          <EuiContextMenuItem icon="link" onClick={addTrustAnchorClicked}>
            Add trust anchor
          </EuiContextMenuItem>
          {import.meta.env.VITE_GIT_SHA && maybeGithubRepoName && (
            <EuiContextMenuItem size="s">
              <EuiSpacer size="s" />
              <EuiText size="xs" textAlign="right" color="grey">
                built from:{" "}
                <EuiLink
                  target="_blank"
                  href={`https://github.com/${maybeGithubRepoName}/commit/${import.meta.env.VITE_GIT_SHA}`}
                  style={{ color: "grey" }}
                >
                  {import.meta.env.VITE_GIT_SHA?.substring(0, 7) || "DEV"}
                </EuiLink>
              </EuiText>
            </EuiContextMenuItem>
          )}
        </EuiContextMenuPanel>
      </EuiPopover>
      {flyout}
      {burstCoverMessagesModal}
      {trustedKeyDigestModal}
      {addTrustAnchorModal}
    </Fragment>
  );
};
