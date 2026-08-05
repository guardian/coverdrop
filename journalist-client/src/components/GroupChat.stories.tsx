import type { Meta, StoryObj } from "@storybook/react-vite";
import { GroupChat } from "./GroupChat.tsx";
import { SentinelProfile } from "../model/bindings/SentinelProfile.ts";
import moment from "moment";
import { GroupWithComputed } from "../state/groups.ts";
import { GroupMessage } from "../model/bindings/GroupMessage.ts";

const you: SentinelProfile = {
  id: "you",
  display_name: "YOU",
};

const meta = {
  component: GroupChat,
} satisfies Meta<typeof GroupChat>;

export default meta;

type Story = StoryObj<typeof meta>;

const group_id = "test";

const fakeNow = "29 June 2026 10:00 UTC";

const messages: GroupMessage[] = [
  // TODO probably need a GroupInfo message first to make it more realistic
  {
    id: "one",
    sender: "sentinel-bob",
    read: true,
    group_id,
    published_at: "05 October 2011 14:48 UTC",
    content: {
      Text: "hello all",
    },
  },
  {
    id: "two",
    sender: you.id,
    read: true,
    group_id,
    published_at: moment(fakeNow).subtract(14, "seconds").toISOString(),
    content: {
      Text: "hello Bob",
    },
  },
  {
    id: "three",
    sender: "sentinel-anna",
    read: true,
    group_id,
    published_at: moment(fakeNow).subtract(2, "minutes").toISOString(),
    content: {
      Text: "good to see everyone here",
    },
  },
  {
    id: "four",
    sender: "sentinel-bob",
    read: true,
    group_id,
    published_at: moment(fakeNow).subtract(95, "seconds").toISOString(),
    content: {
      Text: "welcome Anna",
    },
  },
  {
    id: "five",
    sender: you.id,
    read: true,
    group_id,
    published_at: moment(fakeNow).subtract(70, "seconds").toISOString(),
    content: {
      Text: "shall we start the briefing?",
    },
  },
  {
    id: "six",
    sender: "sentinel-carl",
    read: true,
    group_id,
    published_at: moment(fakeNow).subtract(55, "seconds").toISOString(),
    content: {
      Text: "ready when you are\n(this is a multiline message)",
    },
  },
  {
    id: "seven",
    sender: "sentinel-anna",
    read: false,
    group_id,
    published_at: moment(fakeNow).subtract(48, "seconds").toISOString(),
    content: {
      Text: "I have the latest notes\n(ahh yes I can do multiline too)",
    },
  },
  {
    id: "eight",
    sender: "sentinel-bob",
    read: false,
    group_id,
    published_at: moment(fakeNow).subtract(41, "seconds").toISOString(),
    content: {
      Text: "please share them here",
    },
  },
];

const group: GroupWithComputed = {
  id: group_id,
  display_name: "Example group",
  description: "This is an example group chat.",
  members: ["foo", "bar", "baz"],
  messages,
  mostRecentMessage: messages[1],
  unreadCount: messages.reduce(
    (acc, message) => acc + (message.read ? 0 : 1),
    0,
  ),
  otherIdsWhoAreTyping: ["foo", "bar"],
  lastUpdatedTimestamp: messages[1].published_at,
  totalItemCount: messages.length,
};

export const Default: Story = {
  args: {
    key: group.id,
    group,
    sentinelId: you.id,
    close: () => {},
  },
};
