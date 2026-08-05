import { palette } from "../styles/palette.ts";

interface UnreadIndicatorProps {
  unread: number | boolean;
}

export const UnreadIndicator = ({ unread }: UnreadIndicatorProps) =>
  unread ? (
    <div
      style={{
        fontSize: "10px",
        fontWeight: "normal",
        minWidth: "18px",
        height: "18px",
        padding: "0 5px",
        color: "white",
        backgroundColor: palette("chat-sidebar-unread-message-dot-background"),
        borderRadius: "9px",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        transform: "scale(1.1)",
      }}
    >
      {typeof unread === "number" && unread.toLocaleString()}
    </div>
  ) : (
    false
  ); // ternary rather than && here so we don't display zero when unread is a number
