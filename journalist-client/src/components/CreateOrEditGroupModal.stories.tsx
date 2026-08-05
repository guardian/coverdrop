import type { Meta, StoryObj } from "@storybook/react-vite";
import { CreateOrEditGroupModal } from "./CreateOrEditGroupModal.tsx";
import { usePublicInfoStore } from "../state/publicInfo.ts";
import { SentinelProfile } from "../model/bindings/SentinelProfile.ts";

const meta = {
  component: CreateOrEditGroupModal,
} satisfies Meta<typeof CreateOrEditGroupModal>;

export default meta;

type Story = StoryObj<typeof meta>;
type StoryDecorator = Extract<
  NonNullable<Story["decorators"]>,
  unknown[]
>[number];

const you: SentinelProfile = {
  id: "you",
  display_name: "YOU",
};

const sampleContacts: SentinelProfile[] = [
  {
    id: "foo",
    display_name: "Foo",
  },
  you,
  {
    id: "bar",
    display_name: "Bar",
  },
];

const withSampleProfiles: StoryDecorator = (Story) => {
  const publicInfoStore = usePublicInfoStore();
  publicInfoStore.setPublicInfo({
    journalist_profiles: [],
    sentinel_profiles: sampleContacts,
    keys: [],
    default_journalist_id: null,
    max_epoch: Date.now(),
  });
  return <Story />;
};

export const Create: Story = {
  args: {
    close: () => {},
    sentinelId: you.id,
  },
  decorators: [withSampleProfiles],
};

export const Edit: Story = {
  args: {
    close: () => {},
    sentinelId: you.id,
    maybeExistingGroupToEdit: {
      messages: [], // irrelevant
      members: sampleContacts.map(({ id }) => id),
      id: "id",
      display_name: "Sample Existing Group",
      description: "blah blah",
    },
  },
  decorators: [withSampleProfiles],
};
