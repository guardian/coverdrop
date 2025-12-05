import { BackupChecks } from "../model/bindings/BackupChecks";
import { BackupHistoryEntry } from "../model/bindings/BackupHistoryEntry";
import { JournalistIdentity } from "../model/bindings/JournalistIdentity";
import { invokeWithErrorMessage } from "./invokeWithErrorMessage";

export const getBackupChecks = (): Promise<BackupChecks> => {
  return invokeWithErrorMessage("get_backup_checks");
};

export const getShouldRequireBackup = (): Promise<boolean> => {
  return invokeWithErrorMessage("should_require_backup");
};

export const performBackup = (): Promise<void> => {
  return invokeWithErrorMessage("perform_backup");
};

export const ejectBackupVolume = (): Promise<boolean> => {
  return invokeWithErrorMessage("eject_backup_volume");
};

export const getBackupContacts = (): Promise<JournalistIdentity[]> => {
  return invokeWithErrorMessage("get_backup_contacts");
};

export const getBackupHistory = (): Promise<BackupHistoryEntry[]> => {
  return invokeWithErrorMessage("get_backup_history");
};

export const setBackupContacts = (
  contacts: JournalistIdentity[],
): Promise<void> => {
  return invokeWithErrorMessage("set_backup_contacts", { contacts });
};

export const unwrapBackupSecretShare = (
  encryptedShare: string,
): Promise<string> => {
  return invokeWithErrorMessage("unwrap_backup_secret_share", {
    encryptedShare,
  });
};
