import {
  EuiButton,
  EuiButtonIcon,
  EuiContextMenuItem,
  EuiContextMenuPanel,
  EuiPopover,
} from "@elastic/eui";
import { MouseEventHandler } from "react";

interface PerChatMenuProps {
  isOpen: boolean;
  setIsOpen: (isOpen: boolean) => void;
  shouldShowLabel: boolean;
  hasUnread: boolean;
  markAsUnread: () => void;
  isMuted: boolean;
  showEditModal: () => void;
  showMuteModal: () => void;
  showCopyToClipboardModal: () => void;
}

export const PerChatMenu = ({
  isOpen,
  setIsOpen,
  shouldShowLabel,
  hasUnread,
  markAsUnread,
  isMuted,
  showEditModal,
  showMuteModal,
  showCopyToClipboardModal,
}: PerChatMenuProps) => {
  const toggleButtonClick: MouseEventHandler<HTMLButtonElement> = (event) => {
    event.stopPropagation();
    setIsOpen(!isOpen);
  };
  const toggleButton = shouldShowLabel ? (
    <EuiButton
      iconType="arrowDown"
      iconSide="right"
      onClick={toggleButtonClick}
    >
      Options
    </EuiButton>
  ) : (
    <EuiButtonIcon
      iconType={"arrowDown"}
      onClick={toggleButtonClick}
      aria-label="Options"
    />
  );
  return (
    <EuiPopover
      button={toggleButton}
      isOpen={isOpen}
      closePopover={() => setIsOpen(false)}
      anchorPosition="downLeft"
      onClick={(event) => event.stopPropagation()}
    >
      <EuiContextMenuPanel
        size="s"
        onClick={() => setIsOpen(false)}
        items={[
          <EuiContextMenuItem
            key={"markAsUnread"}
            icon={"dot"}
            disabled={hasUnread}
            onClick={markAsUnread}
          >
            Mark as unread
          </EuiContextMenuItem>,
          <EuiContextMenuItem
            key={"updateUserAlias"}
            icon={"pencil"}
            onClick={showEditModal}
          >
            Update user alias
          </EuiContextMenuItem>,
          <EuiContextMenuItem
            key={"muting"}
            icon={isMuted ? "bell" : "bellSlash"}
            onClick={showMuteModal}
          >
            {isMuted ? "Unmute" : "Mute"} source
          </EuiContextMenuItem>,
          <EuiContextMenuItem
            key={"clipboard"}
            icon={"copyClipboard"}
            onClick={showCopyToClipboardModal}
          >
            Copy to clipboard
          </EuiContextMenuItem>,
        ]}
      />
    </EuiPopover>
  );
};
