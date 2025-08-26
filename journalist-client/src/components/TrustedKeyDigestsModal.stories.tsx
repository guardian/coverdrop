import type { Meta, StoryObj } from "@storybook/react-vite";
import { TrustedKeyDigestsModal } from "./TrustedKeyDigestsModal";
import { mockIPC } from "@tauri-apps/api/mocks";
import type { TrustedOrganizationPublicKeyAndDigest } from "../model/bindings/TrustedOrganizationPublicKeyAndDigest";

const mockDigests = [
  {
    pkHex: "1037f9c40656adb2cc469e68758015c2d90f252c94eae10c4bf38c40d25f67b3",
    digest: "raCEIN k0zRAo vmkuGs jHUA",
  },
] satisfies TrustedOrganizationPublicKeyAndDigest[];

const meta = {
  component: TrustedKeyDigestsModal,
} satisfies Meta<typeof TrustedKeyDigestsModal>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    closeModal: () => {},
  },
  beforeEach: () => {
    mockIPC(() => mockDigests);
  },
};
