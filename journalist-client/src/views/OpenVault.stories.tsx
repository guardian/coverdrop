import type { Meta, StoryObj } from "@storybook/react-vite";

import { OpenVault } from "./OpenVault";
import { mockIPC } from "@tauri-apps/api/mocks";
import { VaultState } from "../model/bindings/VaultState";
import { Profiles } from "../model/bindings/Profiles";

const meta = {
  component: OpenVault,
} satisfies Meta<typeof OpenVault>;

export default meta;

const vaultState = {
  type: "VaultState",
  id: "test",
  path: "fake/vault/to/path.vault",
  isSoftLocked: false,
} satisfies VaultState;

const profiles = {
  DEV: { apiUrl: "https://mock-secure-messaging-api.com" },
} satisfies Profiles;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    setVaultState: () => {},
  },
  beforeEach: () => {
    mockIPC((cmd) => {
      switch (cmd) {
        case "get_vault_state":
          return vaultState;
        case "get_profiles":
          return profiles;
        default:
          break;
      }
    });
  },
};
