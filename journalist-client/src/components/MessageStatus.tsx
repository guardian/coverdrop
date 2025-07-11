import { EuiFlexGroup, EuiFlexItem, EuiIcon } from "@elastic/eui";

// If the message is from today, only show the time,
// otherwise show the date and time
export const formatDateTime = (dateTime: string) => {
  const date = new Date(dateTime);
  const today = new Date();
  if (
    date.getDate() === today.getDate() &&
    date.getMonth() === today.getMonth() &&
    date.getFullYear() === today.getFullYear()
  ) {
    return date.toLocaleTimeString();
  } else {
    return date.toLocaleDateString() + " " + date.toLocaleTimeString();
  }
};

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

      <EuiFlexItem>{formatDateTime(props.timestamp)}</EuiFlexItem>
    </EuiFlexGroup>
  );
};
