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
  EuiBadge,
  EuiIcon,
  EuiSkeletonRectangle,
  EuiFieldSearch,
} from "@elastic/eui";
import { useMessageStore } from "../state/messages";
import { SettingsPopover } from "./SettingsPopover";
import { UserStatus } from "../model/bindings/UserStatus";
import { timeAgo } from "@guardian/libs";
import { palette } from "../styles/palette";
import { useUserStore } from "../state/users";
import { useErrorStore } from "../state/errors";
import { PerChatMenu } from "./PerChatMenu";
import { Message } from "../model/bindings/Message";
import { JournalistStatus } from "../model/bindings/JournalistStatus";
import { Toast } from "@elastic/eui/src/components/toast/global_toast_list";
import moment from "moment";
import {
  ExpiringMessageIcon,
  ExpiringMessageUrgency,
  NEAR_EXPIRY_DAYS,
  URGENT_EXPIRY_HOURS,
} from "./ExpiringMessageIcon";

type Chat = {
  alias: string | null;
  displayName: string;
  description: string | null;
  replyKey: string;
  lastMessageTimestamp: string;
  hasUnread: boolean;
  hasMessagesWithCustomExpiry: boolean;
  userStatus: UserStatus;
  lastMessage: {
    message: string;
    messageType: Message["type"];
  };
  expiringMessageUrgency?: ExpiringMessageUrgency;
};

export type ChatsSideBarProps = {
  journalistId: string;
  journalistStatus?: JournalistStatus;
  currentUserReplyKey: string | null;
  setChat: (userReplyKey: string) => void;
  markChatAsUnread: (replyKey: string) => void;
  setMaybeEditModalForReplyKey: (maybeReplyKey: string | null) => void;
  setMaybeMuteModalForReplyKey: (maybeReplyKey: string | null) => void;
  setMaybeCopyToClipboardModalForReplyKey: (replyKey: string | null) => void;
  setMaybeJournalistStatusForModal: (
    newStatus: JournalistStatus | null,
  ) => void;
  openBackupModal: () => void;
  addCustomToast: (toast: Toast) => void;
  removeCustomToast: (toastId: string) => void;
};

const chatsToSideNav = (
  id: string,
  chats: Chat[],
  setChat: (userReplyKey: string) => void,
  currentUserReplyKey: string | null,
  markChatAsUnread: (replyKey: string) => void,
  setMaybeEditModalForReplyKey: (value: string | null) => void,
  setMaybeMuteModalForReplyKey: (value: string | null) => void,
  setMaybeCopyToClipboardModalForReplyKey: (replyKey: string | null) => void,
) => {
  const { euiTheme } = useEuiTheme();
  const { font, size } = euiTheme;

  const [maybeContextMenuOpenForReplyKey, setMaybeContextMenuOpenForReplyKey] =
    useState<string | null>(null);

  return chats.length > 0
    ? chats.map((chat) => {
        const lastMessageEpoch = new Date(chat.lastMessageTimestamp).getTime();
        const lastMessage = chat.lastMessage;
        const name = chat.alias || chat.displayName;
        return {
          id: htmlIdGenerator(id)(),
          name,
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
                onContextMenu={(event) => {
                  event.preventDefault();
                  setMaybeContextMenuOpenForReplyKey(chat.replyKey);
                }}
              >
                <EuiFlexGroup gutterSize="s" alignItems="center">
                  <EuiFlexItem
                    grow={true}
                    style={{ fontWeight: font.weight.bold }}
                    title={chat.description}
                  >
                    {name}
                  </EuiFlexItem>
                  {chat.hasMessagesWithCustomExpiry && (
                    <EuiIcon
                      type="clockCounter"
                      color="primary"
                      size="m"
                      title="Some messages in this chat have custom expiries."
                    />
                  )}
                  {chat.hasUnread && (
                    <div>
                      <EuiFlexItem
                        style={{
                          width: size.s,
                          height: size.s,
                          backgroundColor: palette(
                            "chat-sidebar-unread-message-dot-background",
                          ),
                          borderRadius: "50%",
                          marginBottom: "1px",
                          display: "inline-block",
                        }}
                      />
                    </div>
                  )}
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
                <EuiFlexGroup gutterSize="s" alignItems="center">
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
                    {lastMessage.messageType === "journalistToUserMessage" &&
                      "You: "}
                    {lastMessage.message}
                  </EuiFlexItem>
                  {chat.expiringMessageUrgency && (
                    <EuiFlexItem grow={false}>
                      <ExpiringMessageIcon
                        expiringMessageUrgency={chat.expiringMessageUrgency}
                        context="CHAT_SIDE_BAR"
                      />
                    </EuiFlexItem>
                  )}
                  <EuiFlexItem grow={false}>
                    <PerChatMenu
                      isOpen={chat.replyKey === maybeContextMenuOpenForReplyKey}
                      setIsOpen={(isOpen) =>
                        setMaybeContextMenuOpenForReplyKey(
                          isOpen ? chat.replyKey : null,
                        )
                      }
                      shouldShowLabel={false}
                      hasUnread={chat.hasUnread}
                      markAsUnread={() => markChatAsUnread(chat.replyKey)}
                      isMuted={chat.userStatus === "MUTED"}
                      showEditModal={() =>
                        setMaybeEditModalForReplyKey(chat.replyKey)
                      }
                      showMuteModal={() =>
                        setMaybeMuteModalForReplyKey(chat.replyKey)
                      }
                      showCopyToClipboardModal={() =>
                        setMaybeCopyToClipboardModalForReplyKey(chat.replyKey)
                      }
                    />{" "}
                    {/* TODO ideally only show when hovering over that menu item */}
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
          name: "No messages",
        },
      ];
};

export const ChatsSideBar = ({
  journalistId,
  journalistStatus,
  currentUserReplyKey: currentUserReplyKey,
  setChat,
  markChatAsUnread,
  setMaybeEditModalForReplyKey,
  setMaybeMuteModalForReplyKey,
  setMaybeCopyToClipboardModalForReplyKey,
  setMaybeJournalistStatusForModal,
  openBackupModal,
  addCustomToast,
  removeCustomToast,
}: ChatsSideBarProps) => {
  const [searchText, setSearchText] = useState("");
  const [isSideNavOpenOnMobile, setIsSideNavOpenOnMobile] = useState(false);

  // Dev mode is for experimental features that aren't ready to be rolled out to everyone.
  // It might make sense to lift this state up to App.tsx if other components need it.
  const [devMode, setDevMode] = useState(false);

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

  const now = moment();
  const chats = Object.values(
    messages.reduce(
      (acc, message) => {
        if (!userInfo[message.userPk]) {
          // error created above
          return acc;
        }
        const thisUserInfo = userInfo[message.userPk];
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

        const expiry = moment(message.customExpiry ?? message.normalExpiry);
        const timeUntilExpiry = moment.duration(expiry.diff(now));
        const hasExpiringMessages = timeUntilExpiry.asDays() < NEAR_EXPIRY_DAYS;
        const hasUrgentlyExpiringMessage =
          timeUntilExpiry.asHours() < URGENT_EXPIRY_HOURS;
        const currentMessageExpiringUrgency: ExpiringMessageUrgency =
          hasUrgentlyExpiringMessage
            ? "URGENT"
            : hasExpiringMessages
              ? "NEAR"
              : undefined;
        const chatExpiringUrgency = maybeExistingInAcc?.expiringMessageUrgency;
        let expiringMessageUrgency: ExpiringMessageUrgency = undefined;
        if (
          currentMessageExpiringUrgency === "URGENT" ||
          chatExpiringUrgency === "URGENT"
        ) {
          expiringMessageUrgency = "URGENT";
        } else if (
          currentMessageExpiringUrgency === "NEAR" ||
          chatExpiringUrgency === "NEAR"
        ) {
          expiringMessageUrgency = "NEAR";
        }
        return {
          ...acc,
          [message.userPk]: {
            alias: thisUserInfo.alias,
            displayName: thisUserInfo.displayName,
            description: thisUserInfo.description,
            replyKey: message.userPk,
            lastMessageTimestamp: isLatestMessage
              ? messageTimestamp
              : maybeExistingInAcc.lastMessageTimestamp,
            hasUnread:
              thisUserInfo.markedAsUnread ||
              !isRead ||
              maybeExistingInAcc?.hasUnread,
            hasMessagesWithCustomExpiry:
              !!message.customExpiry ||
              maybeExistingInAcc?.hasMessagesWithCustomExpiry,
            userStatus: thisUserInfo.status,
            lastMessage: isLatestMessage
              ? { message: message.message, messageType: message.type }
              : maybeExistingInAcc.lastMessage,
            expiringMessageUrgency,
          },
        };
      },
      {} as Record<string, Chat>,
    ),
  )
    .sort((a, b) => (a.lastMessageTimestamp > b.lastMessageTimestamp ? -1 : 1))
    .filter(
      (chat) =>
        !searchText ||
        chat.description?.toLowerCase().includes(searchText.toLowerCase()) ||
        chat.alias?.toLowerCase().includes(searchText.toLowerCase()) ||
        chat.displayName.toLowerCase().includes(searchText.toLowerCase()),
    );

  const inboxChats = chats.filter((c) => c.userStatus == "ACTIVE");
  const mutedChats = chats.filter((c) => c.userStatus == "MUTED");

  const inboxItems = chatsToSideNav(
    "inboxItems",
    inboxChats,
    setChat,
    currentUserReplyKey,
    markChatAsUnread,
    setMaybeEditModalForReplyKey,
    setMaybeMuteModalForReplyKey,
    setMaybeCopyToClipboardModalForReplyKey,
  );
  const mutedItems = chatsToSideNav(
    "mutedItems",
    mutedChats,
    setChat,
    currentUserReplyKey,
    markChatAsUnread,
    setMaybeEditModalForReplyKey,
    setMaybeMuteModalForReplyKey,
    setMaybeCopyToClipboardModalForReplyKey,
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

  // if the user clicks on journalist id 5 times within 1 second, toggle dev mode
  let clickCount = 0;
  let firstClickTime = 0;
  const handleJournalistIdClick = () => {
    const now = Date.now();
    if (now - firstClickTime > 1000) {
      // reset if more than 1 second has passed since first click
      clickCount = 0;
      firstClickTime = now;
    }
    clickCount++;
    if (clickCount >= 5) {
      setDevMode(!devMode);
      clickCount = 0;
      firstClickTime = 0;
      console.log("Dev mode toggled:", !devMode);
    }
  };

  return (
    <>
      <EuiFlexGroup
        style={{
          fontSize: `calc(${size.base} + 2)`,
          fontWeight: font.weight.bold,
          padding: size.s,
          gap: size.m,
          paddingBottom: size.m,
          borderBottom: `1px solid ${palette("chat-sidebar-journalist-name-border-color")}`,
        }}
        alignItems="center"
      >
        <EuiFlexItem grow={false}>
          <SettingsPopover
            journalistId={journalistId}
            journalistStatus={journalistStatus}
            setMaybeJournalistStatusForModal={setMaybeJournalistStatusForModal}
            openBackupModal={openBackupModal}
            addCustomToast={addCustomToast}
            removeCustomToast={removeCustomToast}
            devMode={devMode}
          />
        </EuiFlexItem>
        <EuiFlexItem grow={true}>
          <div
            onClick={handleJournalistIdClick}
            style={{
              userSelect: "none",
              WebkitUserSelect: "none",
              MozUserSelect: "none",
              msUserSelect: "none",
              cursor: "pointer",
            }}
          >
            {journalistId}
          </div>
        </EuiFlexItem>
        {/* Dev mode badge */}
        {devMode && (
          <EuiFlexItem grow={false}>
            <EuiBadge color="primary">Dev Mode</EuiBadge>
          </EuiFlexItem>
        )}
        {/* Journalist status skeleton or badge */}
        {(journalistStatus == "HIDDEN_FROM_UI" ||
          journalistStatus === undefined) && (
          <EuiFlexItem grow={false}>
            <EuiSkeletonRectangle
              width="54.16px"
              height="20px"
              isLoading={journalistStatus === undefined}
              contentAriaLabel="Status pending"
              title="Status pending"
            >
              <EuiBadge
                color={palette("chat-sidebar-hidden-from-ui-color")}
                title="Your profile is hidden in the app. Sources will not be able to start new conversations with you. Conversations that have already started can continue normally."
              >
                Hidden
              </EuiBadge>
            </EuiSkeletonRectangle>
          </EuiFlexItem>
        )}
      </EuiFlexGroup>
      <EuiSpacer size="s" />
      <EuiFieldSearch
        placeholder="Search names, nicknames & descriptions"
        value={searchText}
        onChange={(event) => {
          setSearchText(event.target.value);
        }}
        isClearable={true}
        autoCapitalize="none"
        autoCorrect="off"
        spellCheck="false"
        aria-label="Search chats"
      />
      <EuiSpacer size="s" />
      <EuiTabs size="s">{renderTabs()}</EuiTabs>
      <EuiSpacer size="m" />
      {selectedTabContent}
    </>
  );
};
