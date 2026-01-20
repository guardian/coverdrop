import { JournalistStatus } from "../model/bindings/JournalistStatus";
import { TrustedOrganizationPublicKeyAndDigest } from "../model/bindings/TrustedOrganizationPublicKeyAndDigest";
import { UntrustedKeysAndJournalistProfiles } from "../model/bindings/UntrustedKeysAndJournalistProfiles";
import { invokeWithErrorMessage } from "./invokeWithErrorMessage";
import { LoggingSession } from "../model/bindings/LoggingSession.ts";
import { LogEntry } from "../model/bindings/LogEntry.ts";
import { ask } from "@tauri-apps/plugin-dialog";

export const updateJournalistStatus = (
  newStatus: JournalistStatus,
): Promise<void> => {
  return invokeWithErrorMessage("update_journalist_status", { newStatus });
};

export const forceRotateIdPk = (): Promise<void> => {
  return invokeWithErrorMessage("force_rotate_id_pk");
};

export const forceRotateMsgPk = (): Promise<void> => {
  return invokeWithErrorMessage("force_rotate_msg_pk");
};

export const getPublicInfo =
  (): Promise<UntrustedKeysAndJournalistProfiles | null> => {
    return invokeWithErrorMessage("get_public_info");
  };

export const getLogs = (params: {
  minLevel: string;
  searchTerm: string;
  before: Date;
  limit: number;
  offset: number;
}): Promise<LogEntry[]> => {
  return invokeWithErrorMessage("get_logs", params);
};

export const getLoggingSessionsTimeline = (): Promise<LoggingSession[]> => {
  return invokeWithErrorMessage("get_logging_sessions_timeline");
};

export const getVaultKeys = (): Promise<string> => {
  return invokeWithErrorMessage("get_vault_keys");
};

export const getTrustAnchorDigests = (): Promise<
  TrustedOrganizationPublicKeyAndDigest[]
> => {
  return invokeWithErrorMessage("get_trust_anchor_digests");
};

export const fullyExitApp = async (): Promise<void> => {
  if (
    !(await ask(
      "It's strongly recommended to have Sentinel running. Are you really sure you want to completely exit?",
      {
        title: "Are you sure you want to exit Sentinel?",
        kind: "warning",
        okLabel: "Cancel", // make cancel the primary action
        cancelLabel: "Exit",
      },
    ))
  ) {
    return await invokeWithErrorMessage("fully_exit_app");
  }
};

export const launchNewSentinelInstance = (): Promise<void> => {
  return invokeWithErrorMessage("launch_new_instance");
};
