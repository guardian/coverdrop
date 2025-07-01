import { useEffect, useMemo, useState } from "react";

import {
  EuiFlexItem,
  EuiFlexGroup,
  EuiSideNav,
  htmlIdGenerator,
  EuiSpacer,
  EuiTab,
  EuiTabs,
  EuiHorizontalRule,
  useEuiTheme,
} from "@elastic/eui";
import { useMessageStore } from "../state/messages";
import { SettingsPopover } from "./SettingsPopover";
import { UserStatus } from "../model/bindings/UserStatus";
import { timeAgo } from "@guardian/libs";
import { palette } from "../styles/palette";
import { useUserStore } from "../state/users";
import { useErrorStore } from "../state/errors";

type Chat = {
  name: string;
  replyKey: string;
  lastMessageTimestamp: string;
  hasUnread: boolean;
  userStatus: UserStatus;
  lastMessage: string;
};

type ChatsSideBarProps = {
  userAlias: string;
  currentUserReplyKey: string | null;
  setChat: (userReplyKey: string) => void;
};

const chatsToSideNav = (
  id: string,
  chats: Chat[],
  setChat: (userReplyKey: string) => void,
  currentUserReplyKey: string | null,
) => {
  const { euiTheme } = useEuiTheme();
  const { font, size } = euiTheme;

  return chats.length > 0
    ? chats.map((chat) => {
        const lastMessageEpoch = new Date(chat.lastMessageTimestamp).getTime();
        const lastMessage = chat.lastMessage;
        return {
          id: htmlIdGenerator(id)(),
          name: chat.name,
          isSelected: chat.replyKey === currentUserReplyKey,
          style: {
            marginTop: "0px", // remove default margin on euiSideNavItem
            paddingTop: "0px", // remove default padding on euiSideNavItem
          },
          renderItem: () => (
            <>
              <div
                className="euiSideNavItemButton__content"
                dir="row"
                onClick={() => setChat(chat.replyKey)}
                style={{
                  gap: size.s,
                  borderRadius: size.xs,
                  padding: size.xs,
                  background:
                    chat.replyKey === currentUserReplyKey
                      ? palette("chat-sidebar-selected-chat-background")
                      : palette("chat-sidebar-unselected-chat-background"),
                  cursor: "pointer",
                }}
              >
                <EuiFlexGroup justifyContent="spaceBetween">
                  <EuiFlexItem
                    grow={false}
                    style={{ fontWeight: font.weight.bold }}
                  >
                    {chat.name}
                  </EuiFlexItem>
                  <EuiFlexItem
                    grow={false}
                    style={{
                      color: chat.hasUnread
                        ? palette("chat-sidebar-unread-message-time-color")
                        : "default",
                    }}
                  >
                    {timeAgo(lastMessageEpoch)}
                  </EuiFlexItem>
                </EuiFlexGroup>
                <EuiSpacer size="s" />
                <EuiFlexGroup alignItems="center">
                  <EuiFlexItem
                    style={{
                      height: size.base,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                      bold: "true",
                      color: palette("chat-sidebar-message-preview-color"),
                    }}
                    grow={true}
                  >
                    {lastMessage}
                  </EuiFlexItem>
                  <EuiFlexItem grow={false}>
                    <div>
                      {chat.hasUnread ? (
                        <EuiFlexItem
                          style={{
                            width: size.s,
                            height: size.s,
                            backgroundColor: palette(
                              "chat-sidebar-unread-message-dot-background",
                            ),
                            borderRadius: "50%",
                            display: "inline-block",
                          }}
                        />
                      ) : (
                        <EuiFlexItem grow={false} />
                      )}
                    </div>
                  </EuiFlexItem>
                </EuiFlexGroup>
              </div>
              <EuiHorizontalRule margin="xs" />
            </>
          ),
        };
      })
    : [
        {
          id: htmlIdGenerator(id)(),
          name: "No messages yet",
        },
      ];
};

export const ChatsSideBar = ({
  userAlias,
  currentUserReplyKey: currentUserReplyKey,
  setChat,
}: ChatsSideBarProps) => {
  const [isSideNavOpenOnMobile, setIsSideNavOpenOnMobile] = useState(false);
  const messageStore = useMessageStore();
  const userStore = useUserStore();

  const { euiTheme } = useEuiTheme();
  const { font, size } = euiTheme;

  const toggleOpenOnMobile = () => {
    setIsSideNavOpenOnMobile(!isSideNavOpenOnMobile);
  };

  const messages = messageStore.messages;
  const messageUserPks = new Set(messages.map((m) => m.userPk));
  const userInfo = userStore.getUserInfo();

  useEffect(() => {
    messageUserPks.forEach((messageUserPk) => {
      if (!userInfo[messageUserPk]) {
        console.warn(
          `User info for userPk ${messageUserPk} not found in userInfo store`,
        );
        useErrorStore
          .getState()
          .addError(`User info for ${messageUserPk} not found.`);
      }
    });
  }, [JSON.stringify(messageUserPks), JSON.stringify(userInfo)]);

  const chats = Object.values(
    messages.reduce(
      (acc, message) => {
        if (!userInfo[message.userPk]) {
          // error created above
          return acc;
        }
        const thisUserInfo = userInfo[message.userPk];
        const chatName = thisUserInfo.alias || thisUserInfo.displayName;
        const maybeExistingInAcc = acc[message.userPk];
        const messageTimestamp =
          message.type === "userToJournalistMessage"
            ? message.receivedAt
            : message.sentAt;
        const isRead =
          message.type === "userToJournalistMessage" ? message.read : true;
        const isLatestMessage =
          !maybeExistingInAcc ||
          maybeExistingInAcc.lastMessageTimestamp < messageTimestamp;
        return {
          ...acc,
          [message.userPk]: {
            name: chatName,
            replyKey: message.userPk,
            lastMessageTimestamp: isLatestMessage
              ? messageTimestamp
              : maybeExistingInAcc.lastMessageTimestamp,
            hasUnread: !isRead || maybeExistingInAcc?.hasUnread,
            userStatus: thisUserInfo.status,
            lastMessage: isLatestMessage
              ? message.message
              : maybeExistingInAcc.lastMessage,
          },
        };
      },
      {} as Record<string, Chat>,
    ),
  ).sort((a, b) => (a.lastMessageTimestamp > b.lastMessageTimestamp ? -1 : 1));

  const inboxChats = chats.filter((c) => c.userStatus == "ACTIVE");
  const mutedChats = chats.filter((c) => c.userStatus == "MUTED");

  const inboxItems = chatsToSideNav(
    "inboxItems",
    inboxChats,
    setChat,
    currentUserReplyKey,
  );
  const mutedItems = chatsToSideNav(
    "mutedItems",
    mutedChats,
    setChat,
    currentUserReplyKey,
  );

  const tabs = [
    {
      id: "inbox",
      name: "Inbox",
      content: (
        <EuiSideNav
          aria-label="inbox"
          mobileTitle="Inbox"
          toggleOpenOnMobile={() => toggleOpenOnMobile()}
          isOpenOnMobile={isSideNavOpenOnMobile}
          items={inboxItems}
        />
      ),
    },
    {
      id: "mutedChats",
      name: "Muted",
      content: (
        <EuiSideNav
          aria-label="mutedChats"
          mobileTitle="Muted Chats"
          toggleOpenOnMobile={() => toggleOpenOnMobile()}
          isOpenOnMobile={isSideNavOpenOnMobile}
          items={mutedItems}
        />
      ),
    },
  ];

  const [selectedTabId, setSelectedTabId] = useState("inbox");
  const selectedTabContent = useMemo(() => {
    return tabs.find((obj) => obj.id === selectedTabId)?.content;
  }, [selectedTabId, tabs]);

  const handleTabClick = (tabId: string) => {
    setSelectedTabId(tabId);
    setChat("");
  };

  const renderTabs = () => {
    return tabs.map((tab, index) => (
      <EuiTab
        key={index}
        onClick={() => handleTabClick(tab.id)}
        isSelected={tab.id === selectedTabId}
      >
        {tab.name}
      </EuiTab>
    ));
  };

  return (
    <>
      <EuiFlexGroup
        style={{
          fontSize: `calc(${size.base} + 2)`,
          fontWeight: font.weight.bold,
          padding: size.xs,
          borderBottom: `1px solid ${palette("chat-sidebar-journalist-name-border-color")}`,
        }}
      >
        <EuiFlexItem grow={true}>{userAlias}</EuiFlexItem>
        <EuiFlexItem grow={false}>
          <SettingsPopover />
        </EuiFlexItem>
      </EuiFlexGroup>
      <EuiSpacer size="s" />
      <EuiTabs size="s">{renderTabs()}</EuiTabs>
      <EuiSpacer size="m" />
      {selectedTabContent}
    </>
  );
};
