import type { Meta, StoryObj } from "@storybook/react-vite";
import { IsTypingIndicator } from "./IsTypingIndicator.tsx";

const meta = {
  component: IsTypingIndicator,
} satisfies Meta<typeof IsTypingIndicator>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Single: Story = {
  args: {
    idsWhoAreTyping: ["Foo"],
  },
  decorators: [
    (Story) => {
      return <Story />;
    },
  ],
};

export const Multiple: Story = {
  args: {
    idsWhoAreTyping: ["Foo", "Bar", "Baz", "Quux"],
  },
  decorators: [
    (Story) => {
      return (
        <div style={{ maxWidth: "300px" }}>
          <Story />
        </div>
      );
    },
  ],
};
