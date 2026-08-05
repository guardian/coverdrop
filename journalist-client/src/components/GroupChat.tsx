import {
  EuiButton,
  EuiFlexGroup,
  EuiFlexItem,
  EuiIcon,
  EuiText,
  EuiTextArea,
  useEuiTheme,
} from "@elastic/eui";
import { SentinelProfilesDisplay } from "./SentinelProfileDisplay.tsx";
import { GroupMessageDisplay } from "./GroupMessageDisplay.tsx";
import { sizes } from "../styles/sizes.ts";
import { Fragment, useEffect, useMemo, useRef, useState } from "react";
import {
  updateGroupMessageReadStatus,
  sendGroupMessage,
  setIsTypingInGroup,
} from "../commands/groups.ts";
import { GroupWithComputed } from "../state/groups.ts";
import { IsTypingIndicator } from "./IsTypingIndicator.tsx";
import { palette } from "../styles/palette.ts";
import { CreateOrEditGroupModal } from "./CreateOrEditGroupModal.tsx";

interface GroupChatProps {
  /**
   * NOTE: we require `key` to force the parent component to specify one, so that
   * the component gets unmounted and mounted rather than just the group prop changing
   * https://legacy.reactjs.org/blog/2018/06/07/you-probably-dont-need-derived-state.html#recommendation-fully-uncontrolled-component-with-a-key
   * */
  key: string;
  group: GroupWithComputed;
  sentinelId: string;
  close: () => void;
}

export const GroupChat = ({
  group: {
    messages,
    id,
    members,
    description,
    display_name,
    otherIdsWhoAreTyping,
    unreadCount,
  },
  sentinelId,
  close,
}: GroupChatProps) => {
  const { euiTheme } = useEuiTheme();
  const { size } = euiTheme;

  const [isEditingGroup, setIsEditingGroup] = useState(false);

  const [currentMessageDraft, setCurrentMessageDraft] = useState<string | null>(
    null, // rather than "" so we can distinguish the initial mount vs subsequent typing in the useEffect below
  );

  const isTypingTimeout = useRef<NodeJS.Timeout | null>(null);
  useEffect(() => {
    if (currentMessageDraft !== null) {
      if (currentMessageDraft) {
        if (isTypingTimeout.current) {
          // continuing to type, so clear the timeout ready for the next timeout
          clearTimeout(isTypingTimeout.current);
        } else {
          // must've just started typing (because no existing timeout) so notify server
          setIsTypingInGroup(id, true);
        }
        // set up a new 15s timeout (which, if not cleared will notify the server that no longer typing)
        isTypingTimeout.current = setTimeout(() => {
          setIsTypingInGroup(id, false);
          isTypingTimeout.current = null;
        }, 15_000);
      } else if (isTypingTimeout.current) {
        // empty text box and existing timeout
        // promptly notify the server that no longer typing
        setIsTypingInGroup(id, false);
        // clear timeout avoid repeated send of 'no longer typing'
        clearTimeout(isTypingTimeout.current);
        isTypingTimeout.current = null;
      }
    }
  }, [currentMessageDraft]);
  useEffect(
    () => () => {
      // unmounting
      if (isTypingTimeout.current) {
        setIsTypingInGroup(id, false);
        clearTimeout(isTypingTimeout.current);
        isTypingTimeout.current = null;
      }
    },
    [],
  );

  const [markingAsReadStartingTimeouts, setMarkingAsReadTimeouts] = useState<{
    [messageId: string]: NodeJS.Timeout | undefined;
  }>({});

  const countOfMyOwnMessages = useMemo(
    () => messages.filter((_) => _.sender === sentinelId).length,
    [messages, sentinelId],
  );

  const [maybeScrollableElement, setScrollableElement] =
    useState<HTMLElement>();
  const intersectionObserver = useMemo(
    () =>
      maybeScrollableElement &&
      new IntersectionObserver(
        (entries) =>
          entries.forEach((entry) => {
            const { messageId, isUnread } = (entry.target as HTMLElement)
              .dataset;
            if (!messageId || !isUnread) {
              return;
            }
            setMarkingAsReadTimeouts((prevTimeouts) => {
              clearTimeout(prevTimeouts[messageId]);
              return {
                ...prevTimeouts,
                [messageId]: entry.isIntersecting
                  ? setTimeout(() => {
                      // NOTE: no need to 'unobserve' here since we check for unread on the element further down and
                      // in fact unobserving can cause it to get observed again before the new 'read' status has come back
                      setMarkingAsReadTimeouts((prev) => ({
                        ...prev,
                        [messageId]: undefined,
                      }));
                      if (
                        isUnread === "true" &&
                        document.body.contains(entry.target)
                      ) {
                        const allUnreadIds = Array.from(
                          entry.target.parentElement!.children,
                        ).reduce((acc, element) => {
                          const { messageId, isUnread } = (
                            element as HTMLElement
                          ).dataset;
                          return messageId && isUnread === "true"
                            ? [...acc, messageId]
                            : acc;
                        }, [] as string[]);
                        // TODO this is a little aggressive on the DB and ideally we would throttle these when scrolling quickly
                        updateGroupMessageReadStatus(
                          id,
                          allUnreadIds.slice(
                            0,
                            allUnreadIds.indexOf(entry.target.id) + 1,
                          ),
                          true,
                        );
                      }
                    }, 2000)
                  : undefined,
              };
            });
          }),
        {
          root: maybeScrollableElement,
          rootMargin: "0px",
          threshold: 1.0,
        },
      ),
    [maybeScrollableElement],
  );
  useEffect(() => {
    return () => intersectionObserver?.disconnect();
  }, [intersectionObserver]);
  const scrollToJustBelowUnreadDivider = () => {
    if (maybeScrollableElement) {
      const maybeUnreadDivider = maybeScrollableElement.querySelector(
        "#unread-divider",
      ) as HTMLElement;
      const maybeElementAfterDivider =
        maybeUnreadDivider &&
        (maybeScrollableElement.childNodes.item(
          Array.from(maybeScrollableElement.childNodes).indexOf(
            maybeUnreadDivider,
          ) + 1,
        ) as HTMLElement);
      if (maybeElementAfterDivider) {
        maybeScrollableElement.scrollTo({
          behavior: "smooth",
          // peek at the top of item below the divider, to encourage user to scroll down
          top:
            maybeElementAfterDivider.offsetTop -
            maybeScrollableElement.offsetTop -
            maybeScrollableElement.offsetHeight +
            10,
        });
      }
    }
  };
  useEffect(scrollToJustBelowUnreadDivider, [maybeScrollableElement]); // just once when the scrollable area is mounted
  useEffect(
    () => {
      if (
        maybeScrollableElement &&
        !maybeScrollableElement.querySelector("#unread-divider")
      ) {
        maybeScrollableElement?.lastElementChild?.scrollIntoView({
          behavior: "smooth",
          block: "end",
        });
      }
    },
    // on mount and then whenever the list of messages from ourselves changes, i.e. send
    [maybeScrollableElement, countOfMyOwnMessages],
  );

  const sendMessage = async () => {
    if (currentMessageDraft) {
      await sendGroupMessage(id, currentMessageDraft);
      setCurrentMessageDraft(""); // note this will trigger the useEffect for isTyping
    }
  };

  // not using the members lists as messages could've been sent by someone who has now left
  const uniqueSenderIds = useMemo(
    () => Array.from(new Set(messages.map((_) => _.sender))),
    [messages],
  );

  return (
    <EuiFlexGroup
      direction="column"
      gutterSize="none"
      style={{
        height: "100vh",
        zIndex: 0, // establish predictable stacking context
      }}
    >
      {isEditingGroup && (
        <CreateOrEditGroupModal
          close={() => setIsEditingGroup(false)}
          sentinelId={sentinelId}
          maybeExistingGroupToEdit={{
            id,
            messages,
            members,
            description,
            display_name,
          }}
        />
      )}
      <div style={{ padding: size.m, borderBottom: "2px solid grey" }}>
        <EuiFlexGroup>
          <EuiFlexItem grow>
            <EuiText style={{ fontWeight: "bold" }}>{display_name}</EuiText>
            <EuiText size="s">{description}</EuiText>
            <div>
              <EuiIcon type="users" size="l" />{" "}
              <em>
                <SentinelProfilesDisplay ids={members} />
              </em>
            </div>
          </EuiFlexItem>
          <EuiButton iconType="gear" onClick={() => setIsEditingGroup(true)}>
            Edit Group
          </EuiButton>
        </EuiFlexGroup>
      </div>
      <EuiFlexItem grow={true} style={{ minHeight: 0 }}>
        <EuiFlexGroup
          ref={setScrollableElement}
          direction="column"
          gutterSize="s"
          style={{
            margin: size.m,
            height: "100%",
            overflowY: "scroll", // Make messages scrollable
            overflowX: "visible", // important for horizontal popover room
            zIndex: 0, // establish predictable stacking context
            paddingBottom: size.base, // Add some padding at the bottom
          }}
        >
          {messages
            ?.filter((_) => !("IsTyping" in _.content))
            .map((message, index, messages) => (
              <Fragment key={message.id}>
                {!message.read &&
                  (index === 0 || messages[index - 1]?.read) && (
                    <div
                      id="unread-divider"
                      key="unread-divider"
                      style={{
                        background: euiTheme.colors.emptyShade,
                        position: "sticky",
                        bottom: "-16px",
                        cursor: "pointer",
                      }}
                      onClick={scrollToJustBelowUnreadDivider}
                    >
                      <EuiFlexGroup
                        alignItems="center"
                        gutterSize="s"
                        style={{
                          color: palette(
                            "chat-sidebar-unread-message-dot-background",
                          ),
                          flexGrow: 0,
                        }}
                      >
                        <EuiFlexItem grow>
                          <hr
                            style={{
                              borderColor: palette(
                                "chat-sidebar-unread-message-dot-background",
                              ),
                              borderWidth: "0.1px",
                            }}
                          />
                        </EuiFlexItem>
                        <em>
                          {unreadCount} unread{" "}
                          {unreadCount === 1 ? "item" : "items"} below 👇
                        </em>
                        <EuiFlexItem grow>
                          <hr
                            style={{
                              borderColor: palette(
                                "chat-sidebar-unread-message-dot-background",
                              ),
                              borderWidth: "0.1px",
                            }}
                          />
                        </EuiFlexItem>
                      </EuiFlexGroup>
                    </div>
                  )}
                <div
                  id={message.id}
                  data-message-id={message.id}
                  data-is-unread={!message.read}
                  ref={(element) =>
                    element &&
                    intersectionObserver?.[
                      message.read ? "unobserve" : "observe"
                    ](element)
                  }
                >
                  <GroupMessageDisplay
                    sentinelId={sentinelId}
                    message={message}
                    uniqueSenderIndex={uniqueSenderIds.indexOf(message.sender)}
                    isBeingMarkedAsRead={
                      !!markingAsReadStartingTimeouts[message.id]
                    }
                    historyUpThisPoint={messages.slice(0, index)}
                    markUnreadFromHere={() => {
                      updateGroupMessageReadStatus(
                        id,
                        messages.slice(index).map((_) => _.id),
                        false,
                      );
                      close();
                    }}
                  />
                </div>
              </Fragment>
            ))}
        </EuiFlexGroup>
      </EuiFlexItem>
      {members.includes(sentinelId) ? (
        <EuiFlexItem grow={false}>
          <div style={{ textAlign: "center" }}>
            <IsTypingIndicator idsWhoAreTyping={otherIdsWhoAreTyping} />
          </div>
          <EuiFlexGroup
            gutterSize="s"
            style={{
              padding: size.m,
            }}
          >
            <EuiFlexItem grow>
              <EuiTextArea
                style={{
                  // @ts-expect-error Hide the focus colour provided by EUI
                  "--euiFormControlStateColor": "transparent",
                  paddingBottom: size.xs,
                  minHeight: sizes.chatInput.minHeight,
                }}
                placeholder="Enter a message..."
                value={currentMessageDraft || ""}
                fullWidth
                resize="none"
                rows={1} /*TODO better would be to auto-resize*/
                onChange={(e) => {
                  const message = e.target.value;
                  setCurrentMessageDraft(message);
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey && !e.altKey) {
                    e.preventDefault();
                    e.stopPropagation();
                    sendMessage();
                  }
                }}
                autoCapitalize="off"
                autoCorrect="off"
                autoComplete="off"
              />
            </EuiFlexItem>
            <EuiButton
              iconType="launch"
              iconSide="right"
              disabled={!currentMessageDraft}
              onClick={sendMessage}
            >
              Send
            </EuiButton>
          </EuiFlexGroup>
        </EuiFlexItem>
      ) : (
        <EuiText color="accent" style={{ textAlign: "center" }}>
          You are no longer a member of this group.
        </EuiText>
      )}
    </EuiFlexGroup>
  );
};
