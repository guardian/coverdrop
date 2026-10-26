import type { Meta, StoryObj } from "@storybook/react-vite";
import { clearMocks, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import moment from "moment";
import { expect, waitFor, within } from "storybook/test";

import App from "./App";
import { Group } from "./model/bindings/Group";
import { GroupMessage } from "./model/bindings/GroupMessage";
import { Message } from "./model/bindings/Message";
import { UntrustedKeysAndJournalistProfiles } from "./model/bindings/UntrustedKeysAndJournalistProfiles";
import { User } from "./model/bindings/User";
import { useMessageStore } from "./state/messages";
import { usePublicInfoStore } from "./state/publicInfo";
import { useUserStore } from "./state/users";

const journalistId = "journalist-123";
const sentinelId = "sentinel-456";
const userPk1 =
  "700b6e6f19fcf2d88103baf44d9ed097dae3f8c372825d85b74b95c1a90c6b6e";
const userPk2 =
  "b11b6e6f19fcf2d88103baf44d9ed097dae3f8c372825d85b74b95c1a90c6aaa";
const groupChatMemberOne = "sentinel-bob";
const groupChatMemberTwo = "sentinel-anna";
const groupChatMemberThree = "sentinel-carl";

const messageReceiptDate = new Date("2025-08-01 15:00:00").toISOString();
const normalExpiry = moment().add(14, "days").toISOString();
const nearExpiry = moment().add(2, "days").toISOString();

const oneToTwenty = [
  "one",
  "two",
  "three",
  "four",
  "five",
  "six",
  "seven",
  "eight",
  "nine",
  "ten",
  "eleven",
  "twelve",
  "thirteen",
  "fourteen",
  "fifteen",
  "sixteen",
  "seventeen",
  "eighteen",
  "nineteen",
  "twenty",
];

const vaultState = {
  type: "VaultState",
  journalistId,
  sentinelId,
  path: "/tmp/example.vault",
  isSoftLocked: false,
};

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
    displayName: "Another source",
    alias: "Another callout user",
    description: "Description of another fake user",
    markedAsUnread: true,
  },
] satisfies User[];

const messages = [
  {
    type: "userToJournalistMessage",
    id: BigInt(1),
    userPk: userPk1,
    message: "Hi, I'm a very important source. I have a lead for you.",
    receivedAt: messageReceiptDate,
    normalExpiry,
    customExpiry: null,
    read: false,
    authenticated: true,
  },
  {
    type: "journalistToUserMessage",
    id: BigInt(2),
    userPk: userPk1,
    message: "Thanks — please tell me as much as you can.",
    normalExpiry,
    customExpiry: null,
    isSent: true,
    sentAt: messageReceiptDate,
  },
  {
    type: "userToJournalistMessage",
    id: BigInt(3),
    userPk: userPk2,
    message: "This is a test message, I have something important to reveal...",
    receivedAt: moment(messageReceiptDate).subtract(2, "hours").toISOString(),
    normalExpiry: nearExpiry,
    customExpiry: null,
    read: true,
    authenticated: true,
  },
  {
    type: "userToJournalistMessage",
    id: BigInt(4),
    userPk: userPk2,
    message: "I want to share more detail about the incident.",
    receivedAt: moment(messageReceiptDate).subtract(5, "minutes").toISOString(),
    normalExpiry,
    customExpiry: null,
    read: false,
    authenticated: true,
  },
] satisfies Message[];

const publicInfo: UntrustedKeysAndJournalistProfiles = {
  journalist_profiles: [
    {
      id: journalistId,
      display_name: "Test Desk",
      sort_name: "Desk",
      description: "Witness desk",
      is_desk: true,
      tag: "desk",
      status: "VISIBLE",
    },
  ],
  default_journalist_id: journalistId,
  keys: [],
  max_epoch: 1,
  sentinel_profiles: [{ id: sentinelId, display_name: "Sentinel One" }],
};

const meta = {
  component: App,
  tags: ["screenshot-ready"],
  parameters: {
    layout: "fullscreen",
  },
} satisfies Meta<typeof App>;

export default meta;

type Story = StoryObj<typeof meta>;

const longConversationUserPk =
  "0b4d2b3d4d9b4d2cbeef42c6b6f33d88d30f9d1a9d5e8b4d5d5ce6d9a7c11f";

const longConversationMessages: Message[] = Array.from(
  { length: oneToTwenty.length },
  (_, index) => {
    const ordinal = oneToTwenty[index];

    const isJournalist = index % 2 === 0;
    const timestamp = moment(messageReceiptDate)
      .subtract(20 - index, "minutes")
      .toISOString();

    return isJournalist
      ? ({
          type: "journalistToUserMessage",
          id: BigInt(index + 100),
          userPk: longConversationUserPk,
          message: `test ${ordinal}`,
          normalExpiry,
          customExpiry: null,
          isSent: true,
          sentAt: timestamp,
        } satisfies Message)
      : ({
          type: "userToJournalistMessage",
          id: BigInt(index + 100),
          userPk: longConversationUserPk,
          message: `test ${ordinal}`,
          receivedAt: timestamp,
          normalExpiry,
          customExpiry: null,
          read: true,
          authenticated: true,
        } satisfies Message);
  },
);

const longConversationUser: User = {
  type: "User",
  userPk: longConversationUserPk,
  status: "ACTIVE",
  displayName: "Long-running source",
  alias: "Long chat user",
  description: "A chat with twenty numbered test messages.",
  markedAsUnread: false,
};

const longConversationSeed = {
  users: [longConversationUser],
  messages: [...longConversationMessages],
  publicInfo: {
    ...publicInfo,
    journalist_profiles: publicInfo.journalist_profiles.map((profile) => ({
      ...profile,
      status: "VISIBLE",
    })),
  },
} satisfies {
  users: User[];
  messages: Message[];
  publicInfo: UntrustedKeysAndJournalistProfiles;
};

const longGroupId = "group-chat-scroll-test";
const longGroupMessages: GroupMessage[] = Array.from(
  { length: oneToTwenty.length },
  (_, index) => {
    const ordinal = oneToTwenty[index];

    const sender =
      index % 3 === 0
        ? groupChatMemberOne
        : index % 3 === 1
          ? groupChatMemberTwo
          : groupChatMemberThree;

    const publishedAt = moment(messageReceiptDate)
      .subtract(40 - index, "minutes")
      .toISOString();

    return {
      id: `group-message-${index + 1}`,
      sender,
      group_id: longGroupId,
      content: {
        Text: `group test ${ordinal}`,
      },
      read: true,
      published_at: publishedAt,
    } satisfies GroupMessage;
  },
);

const longGroupSeed: Group[] = [
  {
    id: longGroupId,
    display_name: "Scroll / briefing group",
    description: "A large MLS conversation used to test scrolling behaviour.",
    members: [
      sentinelId,
      groupChatMemberOne,
      groupChatMemberTwo,
      groupChatMemberThree,
    ],
    messages: longGroupMessages,
  },
];

const manyChatUsers = Array.from({ length: 11 }, (_, index) => {
  const userPk = `9${index + 2}c6f1${index + 1}9f3d2b5a8f1e4d9c7b3a5e2d4f6a8c9${index + 1}`;

  return {
    type: "User",
    userPk,
    status: "ACTIVE",
    displayName: `Secondary source ${index + 1}`,
    alias: `Secondary chat ${index + 1}`,
    description: `A shorter conversation used to fill the left sidebar (${index + 1}).`,
    markedAsUnread: index % 3 === 0,
  } satisfies User;
});

const manyChatMessages = manyChatUsers.flatMap((user, index) => {
  const baseId = 200 + index * 10;
  const olderTimestamp = moment(messageReceiptDate)
    .subtract(12 - index, "hours")
    .toISOString();

  return [
    {
      type: "userToJournalistMessage",
      id: BigInt(baseId),
      userPk: user.userPk,
      message: `This is a short message from ${user.alias}.`,
      receivedAt: olderTimestamp,
      normalExpiry,
      customExpiry: null,
      read: true,
      authenticated: true,
    } satisfies Message,
    {
      type: "journalistToUserMessage",
      id: BigInt(baseId + 1),
      userPk: user.userPk,
      message: `Thanks, I’ve noted that for follow-up.`,
      normalExpiry,
      customExpiry: null,
      isSent: true,
      sentAt: moment(olderTimestamp).add(3, "minutes").toISOString(),
    } satisfies Message,
  ];
});

const manyChatsSeed = {
  users: [longConversationUser, ...manyChatUsers],
  messages: [...longConversationMessages, ...manyChatMessages],
  publicInfo: {
    ...publicInfo,
    journalist_profiles: publicInfo.journalist_profiles.map((profile) => ({
      ...profile,
      status: "VISIBLE",
    })),
  },
} satisfies {
  users: User[];
  messages: Message[];
  publicInfo: UntrustedKeysAndJournalistProfiles;
};

type Play = NonNullable<Story["play"]>;
const interactionWait = { timeout: 10_000 };

// The screenshot harness knows only this signal, not how to prepare each story.
// Failed interactions must not mark the story ready.
const screenshotReady =
  (interact: Play): Play =>
  async (context) => {
    delete context.canvasElement.dataset.screenshotReady;
    await interact(context);
    context.canvasElement.dataset.screenshotReady = "true";
  };

const selectInboxChat =
  (name: string, message: string): Play =>
  async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await waitFor(() => {
      const sidebar = canvas.getByRole("navigation", { name: "inbox" });
      // Lookup and click together: refreshes can replace sidebar items across awaits.
      within(sidebar).getByText(name, { exact: true }).click();
    }, interactionWait);
    // Scope to the chat panel so a sidebar preview cannot satisfy this check.
    await waitFor(async () => {
      await expect(
        within(canvas.getByRole("main")).getByText(message, { exact: true }),
      ).toBeVisible();
    }, interactionWait);
  };

const selectDefaultChat = selectInboxChat(
  "Callout user",
  "Hi, I'm a very important source. I have a lead for you.",
);
const selectLongChat = selectInboxChat("Long chat user", "test nineteen");

const selectMlsTab: Play = async ({ canvasElement }) => {
  const canvas = within(canvasElement);
  await waitFor(() => {
    canvas.getByRole("tab", { name: /MLS/ }).click();
  }, interactionWait);
  await waitFor(() => {
    const sidebar = canvas.getByRole("navigation", { name: "MLS" });
    within(sidebar)
      .getByText(longGroupSeed[0].display_name, { exact: true })
      .click();
  }, interactionWait);
  await waitFor(async () => {
    await expect(
      within(canvas.getByRole("main")).getByText("group test nineteen", {
        exact: true,
      }),
    ).toBeVisible();
  }, interactionWait);
};

const waitForUpdateLock: Play = async ({ canvasElement }) => {
  await expect(
    await within(canvasElement).findByRole(
      "button",
      { name: "Restart now" },
      interactionWait,
    ),
  ).toBeVisible();
};

const defaultIpcMock = (cmd: string) => {
  switch (cmd) {
    case "get_vault_state":
      return vaultState;
    case "get_users":
      return users;
    case "get_chats":
      return messages;
    case "get_public_info":
      return publicInfo;
    case "get_backup_history":
      return [];
    case "get_groups_and_messages":
      return [];
    case "check_message_length":
      return 0.5;
    default:
      return undefined;
  }
};

const manyChatsIpcMock = (cmd: string) => {
  switch (cmd) {
    case "get_users":
      return manyChatsSeed.users;
    case "get_chats":
      return manyChatsSeed.messages;
    case "get_public_info":
      return manyChatsSeed.publicInfo;
    default:
      return defaultIpcMock(cmd);
  }
};

const longConversationIpcMock = (cmd: string) => {
  switch (cmd) {
    case "get_users":
      return longConversationSeed.users;
    case "get_chats":
      return longConversationSeed.messages;
    case "get_public_info":
      return longConversationSeed.publicInfo;
    default:
      return defaultIpcMock(cmd);
  }
};

const groupChatIpcMock = (cmd: string) => {
  switch (cmd) {
    case "get_groups_and_messages":
      return longGroupSeed;
    default:
      return defaultIpcMock(cmd);
  }
};

const defaultDecorator = (StoryComponent: () => JSX.Element) => {
  const messageStore = useMessageStore();
  const userStore = useUserStore();
  const publicInfoStore = usePublicInfoStore();

  messageStore.setMessages(messages);
  userStore.setUsers(users);
  publicInfoStore.setPublicInfo(publicInfo);

  return <StoryComponent />;
};

const manyChatsDecorator = (StoryComponent: () => JSX.Element) => {
  const messageStore = useMessageStore();
  const userStore = useUserStore();
  const publicInfoStore = usePublicInfoStore();

  messageStore.setMessages(manyChatsSeed.messages);
  userStore.setUsers(manyChatsSeed.users);
  publicInfoStore.setPublicInfo(manyChatsSeed.publicInfo);

  return <StoryComponent />;
};

export const Default: Story = {
  decorators: [defaultDecorator],
  play: screenshotReady(selectDefaultChat),
  beforeEach: () => {
    clearMocks();
    mockWindows("main");
    mockIPC(defaultIpcMock);
  },
};

export const LongConversation: Story = {
  decorators: [
    (StoryComponent) => {
      const messageStore = useMessageStore();
      const userStore = useUserStore();
      const publicInfoStore = usePublicInfoStore();

      messageStore.setMessages(longConversationSeed.messages);
      userStore.setUsers(longConversationSeed.users);
      publicInfoStore.setPublicInfo(longConversationSeed.publicInfo);

      return <StoryComponent />;
    },
  ],
  play: screenshotReady(selectLongChat),
  beforeEach: () => {
    clearMocks();
    mockWindows("main");
    mockIPC(longConversationIpcMock);
  },
};

export const ManyChats: Story = {
  decorators: [manyChatsDecorator],
  play: screenshotReady(selectLongChat),
  beforeEach: () => {
    clearMocks();
    mockWindows("main");
    mockIPC(manyChatsIpcMock);
  },
};

export const GroupConversation: Story = {
  decorators: [defaultDecorator],
  play: screenshotReady(selectMlsTab),
  beforeEach: () => {
    clearMocks();
    mockWindows("main");
    mockIPC(groupChatIpcMock);
  },
};

const buildUpdateAvailableStory = (
  hoursUpdateAvailable: number,
  interact: Play,
  ipcMock: (cmd: string) => unknown,
  seedDecorator = defaultDecorator,
): Story => ({
  decorators: [seedDecorator],
  play: screenshotReady(async (context) => {
    await interact(context);
    if (hoursUpdateAvailable <= 24) {
      await expect(
        await within(context.canvasElement).findByText(
          "New version available!",
          {},
          interactionWait,
        ),
      ).toBeVisible();
    }
  }),
  beforeEach: () => {
    clearMocks();
    mockWindows("main");
    let hasCapturedCreatedTimestampAtLaunch = false;
    mockIPC((cmd) => {
      switch (cmd) {
        case "exit_app":
          return; // do nothing, just in case
        case "get_created_timestamp_of_executable":
          if (!hasCapturedCreatedTimestampAtLaunch) {
            hasCapturedCreatedTimestampAtLaunch = true;
            return moment().subtract(7, "days").toDate();
          }
          return moment().subtract(hoursUpdateAvailable, "hours").toDate();
        default:
          return ipcMock(cmd);
      }
    });
  },
});

export const UpdateAvailableBannerOnUserChat: Story = buildUpdateAvailableStory(
  1,
  selectLongChat,
  manyChatsIpcMock,
  manyChatsDecorator,
); // 1 hour since update available
export const UpdateAvailableBannerOnGroupChat: Story =
  buildUpdateAvailableStory(1, selectMlsTab, groupChatIpcMock); // 1 hour since update available

export const UpdateAvailableUiLock: Story = buildUpdateAvailableStory(
  25,
  waitForUpdateLock,
  defaultIpcMock,
); // 25 hours since update available
