import type { Meta, StoryObj } from "@storybook/react-vite";

import { AddTrustAnchorModal } from "./AddTrustAnchorModal";

const meta = {
  component: AddTrustAnchorModal,
} satisfies Meta<typeof AddTrustAnchorModal>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    closeModal: () => {},
  },
};
