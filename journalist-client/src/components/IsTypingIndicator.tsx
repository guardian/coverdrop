import { EuiBadge, EuiProgress } from "@elastic/eui";
import { SentinelProfileDisplay } from "./SentinelProfileDisplay.tsx";

interface IsTypingIndicatorProps {
  idsWhoAreTyping: string[];
}

export const IsTypingIndicator = ({
  idsWhoAreTyping,
}: IsTypingIndicatorProps) =>
  idsWhoAreTyping.map((id) => (
    <EuiBadge key={id} title="" style={{ position: "relative", margin: "3px" }}>
      <EuiProgress
        size="xs"
        color="subdued"
        position="absolute"
        style={{ bottom: 0 }}
      />
      <SentinelProfileDisplay id={id} /> is typing...
    </EuiBadge>
  ));
