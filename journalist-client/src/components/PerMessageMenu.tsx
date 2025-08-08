import {
  EuiButtonIcon,
  EuiContextMenuItem,
  EuiContextMenuPanel,
  EuiPopover,
} from "@elastic/eui";

interface PerMessageMenuProps {
  isOpen: boolean;
  setIsOpen: (isOpen: boolean) => void;
  openCustomExpiryModal: () => void;
}

export const PerMessageMenu = ({
  isOpen,
  setIsOpen,
  openCustomExpiryModal,
}: PerMessageMenuProps) => (
  <EuiPopover
    button={
      <EuiButtonIcon
        iconType={"arrowDown"}
        onClick={() => setIsOpen(!isOpen)}
        aria-label="Message Options"
      />
    }
    isOpen={isOpen}
    closePopover={() => setIsOpen(false)}
    anchorPosition="downCenter"
    repositionOnScroll
  >
    <EuiContextMenuPanel
      size="s"
      onClick={() => setIsOpen(false)}
      items={[
        <EuiContextMenuItem
          key={"customExpiry"}
          icon={"clockCounter"}
          onClick={openCustomExpiryModal}
        >
          Set custom expiry
        </EuiContextMenuItem>,
      ]}
    />
  </EuiPopover>
);
