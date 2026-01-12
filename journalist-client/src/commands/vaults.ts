import { OpenVaultOutcome } from "../model/bindings/OpenVaultOutcome";
import { VaultState } from "../model/bindings/VaultState";
import { useErrorStore } from "../state/errors";
import {
  invokeWithErrorMessage,
  invokeWithSilencedErrorMessage,
} from "./invokeWithErrorMessage";

export const getVaultState = (): Promise<VaultState | null> => {
  return invokeWithErrorMessage("get_vault_state");
};

export const unlockVault = async (
  profile: string,
  path: string,
  password: string,
): Promise<void> => {
  const outcome = await invokeWithErrorMessage<OpenVaultOutcome>(
    "unlock_vault",
    { profile, path, password },
  );

  if (outcome.type === "openedOffline") {
    useErrorStore
      .getState()
      .addWarning(
        "Vault opened while offline, cannot perform preflight checks",
      );
  }

  if (outcome.type === "openedOnline") {
    for (const missingInVault of outcome.orgPksMissingInVault) {
      useErrorStore
        .getState()
        .addWarning(
          `Organization public key "${missingInVault.substring(0, 8)}" found in API but not in vault. This can cause issues, talk to the admin team.`,
        );
    }

    for (const missingInApi of outcome.orgPksMissingInApi) {
      useErrorStore
        .getState()
        .addWarning(
          `Trust anchor ${missingInApi.substring(0, 8)} found in vault but not in API.`,
        );
    }
  }
};

export const softLockVault = (): Promise<VaultState | null> => {
  return invokeWithErrorMessage("soft_lock_vault");
};

export const unlockSoftLockedVault = (
  password: string,
): Promise<VaultState | null> => {
  return invokeWithErrorMessage("unlock_soft_locked_vault", { password });
};

export const getColocatedPassword = (path: string): Promise<string | null> => {
  return invokeWithSilencedErrorMessage("get_colocated_password", { path });
};

export const addTrustAnchor = (path: string): Promise<string | null> => {
  return invokeWithErrorMessage("add_trust_anchor", { path });
};

export const sendDesktopNotification = (args: {
  title?: string;
  body: string;
}): Promise<void> => {
  return invokeWithErrorMessage("send_notification", args);
};
