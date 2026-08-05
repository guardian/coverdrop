import type { Meta, StoryObj } from "@storybook/react-vite";
import {
  GroupMembersAdded,
  GroupMemberLeft,
  GroupMembersRemoved,
  GroupPropertyChanged,
} from "./GroupChangeMessages.tsx";
import { GroupMessage } from "../model/bindings/GroupMessage.ts";

type GroupChangeMessagesStoryProps =
  | ({
      variant: "added";
    } & Parameters<typeof GroupMembersAdded>[0])
  | ({
      variant: "removed";
    } & Parameters<typeof GroupMembersRemoved>[0])
  | ({
      variant: "left";
    } & Parameters<typeof GroupMemberLeft>[0])
  | ({
      variant: "renamed";
    } & Parameters<typeof GroupPropertyChanged>[0])
  | ({
      variant: "description changed";
    } & Parameters<typeof GroupPropertyChanged>[0]);

const GroupChangeMessagesStory = (props: GroupChangeMessagesStoryProps) => {
  switch (props.variant) {
    case "added":
      return <GroupMembersAdded {...props} />;
    case "removed":
      return <GroupMembersRemoved {...props} />;
    case "left":
      return <GroupMemberLeft {...props} />;
    case "renamed":
    case "description changed":
      return <GroupPropertyChanged {...props} />;
    default:
      throw Error("missing case in story");
  }
};

const meta = {
  component: GroupChangeMessagesStory,
} satisfies Meta<typeof GroupChangeMessagesStory>;

export default meta;

type Story = StoryObj<typeof meta>;

const timestamp = "05 October 2011 14:48 UTC";

// TODO make sure we have fake sentinel profiles loaded

export const MembersAdded: Story = {
  args: {
    timestamp,
    variant: "added",
    addedById: "sentinel-alice",
    idsAdded: ["sentinel-bob", "sentinel-carol", "sentinel-dave"],
  },
};

export const MembersRemoved: Story = {
  args: {
    timestamp,
    variant: "removed",
    removedById: "sentinel-alice",
    idsRemoved: ["sentinel-carol", "sentinel-dave"],
  },
};

export const MemberLeft: Story = {
  args: {
    timestamp,
    variant: "left",
    idWhoLeft: "sentinel-carol",
  },
};

const fakeMessageHistory: GroupMessage[] = [
  // FIXME come up with a comprehensive history covering all types
];

export const GroupWasRenamed: Story = {
  args: {
    timestamp,
    variant: "renamed",
    changeMessageType: "GroupNameChanged",
    changedBy: "sentinel-bob",
    changedTo: "New provocative title",
    historyUpThisPoint: fakeMessageHistory,
  },
};

export const GroupDescriptionChanged: Story = {
  args: {
    timestamp,
    variant: "description changed",
    changeMessageType: "GroupDescriptionChanged",
    changedBy: "sentinel-alice",
    changedTo: "This is a spicy spicy group chat 🌶",
    historyUpThisPoint: fakeMessageHistory,
  },
};
