import { GroupMessage } from "../model/bindings/GroupMessage.ts";
import { SentinelProfileDisplay } from "./SentinelProfileDisplay.tsx";
import {
  GroupMembersAdded,
  GroupMemberLeft,
  GroupMembersRemoved,
  GroupPropertyChanged,
} from "./GroupChangeMessages.tsx";
import { palette } from "../styles/palette.ts";
import { formatDateTimeString } from "../helpers.ts";
import {
  EuiFlexGroup,
  useEuiTheme,
  euiPaletteColorBlind,
  EuiButtonIcon,
  EuiContextMenuPanel,
  EuiContextMenuItem,
  EuiPopover,
} from "@elastic/eui";
import { useState } from "react";

const colourPalette = euiPaletteColorBlind({ rotations: 2, order: "group" });

interface GroupMessageDisplayProps {
  sentinelId: string;
  message: GroupMessage;
  uniqueSenderIndex: number;
  isBeingMarkedAsRead: boolean;
  historyUpThisPoint: GroupMessage[];
  markUnreadFromHere: () => void;
}

export const GroupMessageDisplay = ({
  sentinelId,
  message: { sender, published_at, content, read },
  uniqueSenderIndex,
  isBeingMarkedAsRead,
  historyUpThisPoint,
  markUnreadFromHere,
}: GroupMessageDisplayProps) => {
  const { euiTheme } = useEuiTheme();
  const { size } = euiTheme;

  const timestamp = formatDateTimeString(published_at);

  const isYou = sender === sentinelId;

  const colourPaletteIndex = uniqueSenderIndex * 2;
  const senderColour = colourPalette[colourPaletteIndex];
  const backgroundColour = colourPalette[colourPaletteIndex + 1] + "66"; // 66 is 40% alpha

  const [perMessageMenuIsOpen, setPerMessageMenuIsOpen] = useState(false);

  if ("Text" in content) {
    return (
      <div
        style={{
          textAlign: isYou ? "right" : "left",
        }}
      >
        <EuiFlexGroup
          direction={isYou ? "rowReverse" : "row"}
          alignItems="center"
          gutterSize={"xs"}
        >
          <div
            style={{
              display: "inline-block",
              maxWidth: "85%",
              textAlign: "left",
              backgroundColor: isYou ? "#1133FF" : backgroundColour,
              whiteSpace: "pre-wrap",
              overflowWrap: "break-word",
              color: isYou ? "#EEEEEE" : "#333333",
              padding: size.m,
              width: "fit-content",
              borderRadius: size.s,
            }}
          >
            {!isYou && (
              <div style={{ color: isYou ? "inherit" : senderColour }}>
                <strong>
                  <SentinelProfileDisplay id={sender} />
                </strong>
              </div>
            )}
            <div>{content.Text}</div>
          </div>
          {/* TODO refactor to unread component */}
          {!read && (
            <div
              style={{
                width: size.s,
                height: size.s,
                backgroundColor: palette(
                  "chat-sidebar-unread-message-dot-background",
                ),
                borderRadius: "50%",
                margin: "5px",
                marginBottom: "6px",
                display: "inline-block",
                transition: "opacity 2s linear",
                opacity: isBeingMarkedAsRead ? 0.2 : 1.0,
                willChange: "opacity", // this ensures opacity transition works inside scrollable area
              }}
            />
          )}
          {!isYou && (
            <EuiPopover
              button={
                <EuiButtonIcon
                  iconType={"arrowDown"}
                  onClick={() => setPerMessageMenuIsOpen(!perMessageMenuIsOpen)}
                  aria-label="Message Options"
                />
              }
              isOpen={perMessageMenuIsOpen}
              closePopover={() => setPerMessageMenuIsOpen(false)}
              anchorPosition="downCenter"
              repositionOnScroll
            >
              <EuiContextMenuPanel
                size="s"
                onClick={() => setPerMessageMenuIsOpen(false)}
                items={[
                  <EuiContextMenuItem
                    key="markUnreadFromHere"
                    icon="dot"
                    onClick={markUnreadFromHere}
                  >
                    Mark unread from here
                  </EuiContextMenuItem>,
                ]}
              />
            </EuiPopover>
          )}
        </EuiFlexGroup>
        <div
          style={{
            fontSize: size.m,
            color: palette("message-status-color"),
            marginTop: size.xs,
          }}
        >
          {timestamp}
        </div>
      </div>
    );
  }
  if ("UsersAdded" in content) {
    return (
      <GroupMembersAdded
        timestamp={timestamp}
        idsAdded={content.UsersAdded}
        addedById={sender}
      />
    );
  }
  if (
    "UsersRemoved" in content &&
    content.UsersRemoved.length === 1 &&
    content.UsersRemoved[0] === sender
  ) {
    return <GroupMemberLeft timestamp={timestamp} idWhoLeft={sender} />;
  }
  if ("UsersRemoved" in content) {
    return (
      <GroupMembersRemoved
        timestamp={timestamp}
        idsRemoved={content.UsersRemoved}
        removedById={sender}
      />
    );
  }
  if ("GroupNameChanged" in content) {
    return (
      <GroupPropertyChanged
        timestamp={timestamp}
        changedBy={sender}
        changeMessageType={"GroupNameChanged"}
        changedTo={content.GroupNameChanged}
        historyUpThisPoint={historyUpThisPoint}
      />
    );
  }
  if ("GroupDescriptionChanged" in content) {
    return (
      <GroupPropertyChanged
        timestamp={timestamp}
        changedBy={sender}
        changeMessageType={"GroupDescriptionChanged"}
        changedTo={content.GroupDescriptionChanged ?? ""}
        historyUpThisPoint={historyUpThisPoint}
      />
    );
  }
  if ("IsTyping" in content) {
    // return `${sender} ${content.IsTyping ? "is" : "is not "} typing`;
    return null; // ignore e.g. IsTyping messages are collated and displayed elsewhere
  }
  if ("GroupInfo" in content) {
    return null;
  }
  // THIS IS THE CATCH-ALL - debatable if it should display anything
  return <pre>{JSON.stringify(content, null, "  ")}</pre>;
};
