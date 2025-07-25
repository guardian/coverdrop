import { JournalistStatus } from "../model/bindings/JournalistStatus";
import { SentinelLogEntry } from "../model/bindings/SentinelLogEntry";
import { TrustedOrganizationPublicKeyAndDigest } from "../model/bindings/TrustedOrganizationPublicKeyAndDigest";
import { UntrustedKeysAndJournalistProfiles } from "../model/bindings/UntrustedKeysAndJournalistProfiles";
import { invokeWithErrorMessage } from "./invokeWithErrorMessage";

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
  (): Promise<UntrustedKeysAndJournalistProfiles> => {
    return invokeWithErrorMessage("get_public_info");
  };

export const getLogs = (): Promise<SentinelLogEntry[]> => {
  return invokeWithErrorMessage("get_logs");
};

export const getVaultKeys = (): Promise<string> => {
  return invokeWithErrorMessage("get_vault_keys");
};

export const getTrustAnchorDigests = (): Promise<
  TrustedOrganizationPublicKeyAndDigest[]
> => {
  return invokeWithErrorMessage("get_trust_anchor_digests");
};
