import type { Meta, StoryObj } from "@storybook/react-vite";

import { ChatsSideBar, ChatsSideBarProps } from "./ChatsSideBar";
import { useMessageStore } from "../state/messages";
import { Message } from "../model/bindings/Message";
import { useUserStore } from "../state/users";
import { User } from "../model/bindings/User";
import { ReactNode } from "react";
import { sizes } from "../styles/sizes";

type NarrowWidthWrapperProps = {
  children: ReactNode;
};

// Helper that forces the `ChatsSideBar` component to render in Storybook
// with the same width as it does in the app.
// Otherwise, when rendered in isolation, Storybook expands it to 100% of the viewport,
// which doesn't look like a sidebar at all.
const NarrowWidthWrapper = ({ children }: NarrowWidthWrapperProps) => {
  return (
    <div
      style={{
        width: sizes.chatsSideBar.minWidth,
      }}
    >
      {children}
    </div>
  );
};

const userPk1 =
  "700b6e6f19fcf2d88103baf44d9ed097dae3f8c372825d85b74b95c1a90c6b6e";
const userPk2 =
  "b11b6e6f19fcf2d88103baf44d9ed097dae3f8c372825d85b74b95c1a90c6aaa";

const date = new Date("2025-08-01 15:00:00").toISOString();

const u2jMessage = {
  type: "userToJournalistMessage",
  id: BigInt(10),
  userPk: userPk1,
  message: "Hey there fella",
  receivedAt: date,
  normalExpiry: date,
  customExpiry: null,
  read: true,
} satisfies Message;

const users = [
  {
    type: "User",
    userPk: userPk1,
    status: "ACTIVE",
    displayName: "Fake user",
    alias: "Callout user",
    description: "Description of a fake user",
    markedAsUnread: false,
  },
  {
    type: "User",
    userPk: userPk2,
    status: "ACTIVE",
    displayName: "Fake user",
    alias: "Another callout user",
    description: "Description of another fake user",
    markedAsUnread: true,
  },
] satisfies User[];

const mockU2JMessage = (id: number, userPk: string, message: string) => ({
  ...u2jMessage,
  id: BigInt(id),
  userPk,
  message,
});

const messages = [
  mockU2JMessage(
    1,
    userPk1,
    "This is a test message, I have very important information to reveal...",
  ),
  mockU2JMessage(
    2,
    userPk2,
    "This is another test message, I have some incredibly important information to reveal...",
  ),
] satisfies Message[];

const commonArgs = {
  journalistId: "Test journalist",
  setChat: () => {},
  markChatAsUnread: () => {},
  openBackupModal: () => {},
  setMaybeEditModalForReplyKey: () => {},
  setMaybeMuteModalForReplyKey: () => {},
  setMaybeCopyToClipboardModalForReplyKey: () => {},
  setMaybeJournalistStatusForModal: () => {},
  addCustomToast: () => {},
  removeCustomToast: () => {},
} satisfies Partial<ChatsSideBarProps>;

const meta = {
  component: ChatsSideBar,
} satisfies Meta<typeof ChatsSideBar>;

export default meta;

type Story = StoryObj<typeof meta>;

export const VisibleJournalist: Story = {
  args: {
    ...commonArgs,
    currentUserReplyKey: userPk1,
    journalistStatus: "VISIBLE",
  },
  decorators: [
    (Story) => {
      const messageStore = useMessageStore();
      const userStore = useUserStore();

      userStore.setUsers(users);
      messageStore.setMessages(messages);

      return (
        <NarrowWidthWrapper>
          <Story />
        </NarrowWidthWrapper>
      );
    },
  ],
};

export const HiddenJournalist: Story = {
  args: {
    ...commonArgs,
    currentUserReplyKey: userPk2,
    journalistStatus: "HIDDEN_FROM_UI",
  },
  decorators: [
    (Story) => {
      const messageStore = useMessageStore();
      const userStore = useUserStore();

      userStore.setUsers(users);
      messageStore.setMessages(messages);

      return (
        <NarrowWidthWrapper>
          <Story />
        </NarrowWidthWrapper>
      );
    },
  ],
};

export const NoMessages: Story = {
  args: {
    ...commonArgs,
    currentUserReplyKey: userPk1,
    journalistStatus: "VISIBLE",
  },
  decorators: [
    (Story) => {
      const messageStore = useMessageStore();
      const userStore = useUserStore();

      userStore.setUsers(users);
      messageStore.setMessages([]);

      return (
        <NarrowWidthWrapper>
          <Story />
        </NarrowWidthWrapper>
      );
    },
  ],
};
