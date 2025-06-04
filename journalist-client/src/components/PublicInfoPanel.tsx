import {
  EuiButtonIcon,
  EuiFlyout,
  EuiFlyoutBody,
  EuiFlyoutHeader,
} from "@elastic/eui";

export const PublicInfoPanel = ({
  json,
  setFlyoutVisible,
  refreshClicked,
}: {
  json: string;
  setFlyoutVisible: (visible: boolean) => void;
  refreshClicked: () => void;
}) => {
  return (
    <EuiFlyout size="l" onClose={() => setFlyoutVisible(false)}>
      <EuiFlyoutHeader>
        <EuiButtonIcon
          iconType="refresh"
          onClick={refreshClicked}
        ></EuiButtonIcon>
      </EuiFlyoutHeader>

      <EuiFlyoutBody>
        <div style={{ overflow: "auto" }}>
          <pre>{json}</pre>
        </div>
      </EuiFlyoutBody>
    </EuiFlyout>
  );
};
