import { EuiIcon } from "@elastic/eui";

export const NEAR_EXPIRY_DAYS = 3;
export const URGENT_EXPIRY_HOURS = 24;

export type ExpiringMessageUrgency = "URGENT" | "NEAR" | undefined;
export type Context = "CHAT_SIDE_BAR" | "CHAT";

export const ExpiringMessageIcon = ({
  expiringMessageUrgency,
  context,
}: {
  expiringMessageUrgency: ExpiringMessageUrgency;
  context: Context;
}) => {
  const titleBeginning =
    context === "CHAT_SIDE_BAR" ? "Contains messages that" : "Message";
  return (
    expiringMessageUrgency && (
      <EuiIcon
        type="warning"
        color={expiringMessageUrgency === "URGENT" ? "danger" : "warning"}
        title={
          expiringMessageUrgency === "URGENT"
            ? `${titleBeginning} will expire in the next ${URGENT_EXPIRY_HOURS} hours`
            : `${titleBeginning} will expire in the next ${NEAR_EXPIRY_DAYS} days`
        }
      />
    )
  );
};
