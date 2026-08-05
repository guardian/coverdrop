import { useEffect, useState } from "react";
import { getGroup, getGroups } from "../commands/groups.ts";
import { Group } from "../model/bindings/Group.ts";
import { GroupMessage } from "../model/bindings/GroupMessage.ts";
import moment from "moment";
import { listen } from "@tauri-apps/api/event";
import { GroupId } from "../model/bindings/GroupId.ts";

export interface GroupWithComputed extends Group {
  unreadCount: number;
  totalItemCount: number;
  mostRecentMessage: GroupMessage | undefined;
  otherIdsWhoAreTyping: string[];
  lastUpdatedTimestamp: string;
}

const augmentGroup =
  (sentinelId: string) =>
  (group: Group): GroupWithComputed => {
    const countableMessages = group.messages.filter(
      (_) => !("IsTyping" in _.content || "GroupInfo" in _.content),
    );
    const thirtySecondsAgo = moment().subtract(30, "seconds");
    return {
      ...group,
      lastUpdatedTimestamp: group.messages.reduce(
        (latest, { published_at, content }) =>
          !("IsTyping" in content) && published_at > latest
            ? published_at
            : latest,
        "",
      ),
      unreadCount: countableMessages.filter((_) => !_.read).length,
      totalItemCount: countableMessages.length,
      mostRecentMessage: countableMessages[countableMessages.length - 1],
      otherIdsWhoAreTyping: group.messages.reduce((acc, message) => {
        if (message.sender !== sentinelId && "IsTyping" in message.content) {
          if (
            message.content.IsTyping &&
            // failsafe for whenever somebody loses connection mid-typing so it doesn't show as typing indefinitely
            moment(message.published_at).isAfter(thirtySecondsAgo) &&
            !acc.includes(message.sender)
          ) {
            return [...acc, message.sender];
          } else if (!message.content.IsTyping) {
            return acc.filter((_) => _ !== message.sender);
          }
        }
        return acc;
      }, [] as string[]),
    };
  };

/* This should only be used once, unlike the library wrapped useMessageStore */
export const useGroups = (maybeSentinelId: string | null) => {
  const [maybeGroups, setMaybeGroups] = useState<GroupWithComputed[]>();

  useEffect(() => {
    if (!maybeSentinelId) {
      return;
    }
    const fullRefresh = () =>
      getGroups().then((groups) =>
        setMaybeGroups(groups.map(augmentGroup(maybeSentinelId))),
      );

    fullRefresh();

    const unlistenToGroupChangesFnPromise = listen<GroupId>(
      "group_change",
      ({ payload: groupIdChanged }) =>
        getGroup(groupIdChanged).then((group) => {
          const groupWithComputed = augmentGroup(maybeSentinelId)(group);
          setMaybeGroups((prev) => {
            if (!prev) {
              return [groupWithComputed];
            }
            const indexOfExistingGroupToReplace = prev.findIndex(
              (_) => _.id === groupWithComputed.id,
            );
            if (indexOfExistingGroupToReplace === -1) {
              return [...prev, groupWithComputed];
            }
            return [
              ...prev.slice(0, indexOfExistingGroupToReplace),
              groupWithComputed,
              ...prev.slice(indexOfExistingGroupToReplace + 1),
            ];
          });
        }),
    );

    const intervalPollingForGoodMeasure = setInterval(fullRefresh, 60_000);

    return () => {
      clearInterval(intervalPollingForGoodMeasure);
      unlistenToGroupChangesFnPromise.then((unlisten) => unlisten());
    };
  }, [setMaybeGroups, maybeSentinelId]);

  return {
    maybeGroups,
    groupsUnreadCount:
      maybeGroups?.reduce((acc, group) => acc + group.unreadCount, 0) ?? 0,
  };
};
