// Keep in line with common/src/protocol/constants.rs
// It's not currently possible to export constants via ts-rs, so we re-declare them here.
// See https://github.com/Aleph-Alpha/ts-rs/issues/441

// The minimum number of shares required to reconstruct the backup secret. Must be <= N.
import moment from "moment/moment";

export const SECRET_SHARING_K_VALUE = 1;

// The total number of shares to create for the backup secret. Must be >= K.
export const SECRET_SHARING_N_VALUE = 1;

// Idle timeout duration before soft-locking the vault
export const IDLE_TIMEOUT = moment.duration(1, "hour");
// Duration before idle timeout to show a warning
export const IDLE_WARNING_DURATION = moment.duration(5, "minutes");
