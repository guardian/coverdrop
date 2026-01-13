import { EuiFlexGroup, EuiFlexItem, EuiIcon } from "@elastic/eui";
import { formatDateTimeString } from "../helpers";

export const MessageStatus = (props: {
  isSent: boolean | null;
  timestamp: string;
}) => {
  return (
    <EuiFlexGroup dir="row" gutterSize="xs" alignItems="center">
      {props.isSent === true ? (
        <EuiIcon title="Sent" type="check" size="m" />
      ) : props.isSent === false ? (
        <EuiIcon title="Pending" type="clock" size="m" />
      ) : null}

      <EuiFlexItem>{formatDateTimeString(props.timestamp)}</EuiFlexItem>
    </EuiFlexGroup>
  );
};
