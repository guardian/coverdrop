import {
  PropsWithChildren,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import moment from "moment";
import {
  EuiButton,
  EuiFieldPassword,
  EuiFocusTrap,
  EuiLoadingSpinner,
  EuiOverlayMask,
  EuiSpacer,
  EuiText,
  EuiTitle,
} from "@elastic/eui";
import {
  sendDesktopNotification,
  softLockVault,
  unlockSoftLockedVault,
} from "../commands/vaults.ts";
import { VaultState } from "../model/bindings/VaultState.ts";
import { IDLE_TIMEOUT, IDLE_WARNING_DURATION } from "../constants.ts";

const ONE_DAY_IN_SECONDS = moment.duration(1, "day").asSeconds();

const SECONDS_BEFORE_SOFTLOCK_TO_SHOW_WARNING =
  IDLE_WARNING_DURATION.asSeconds();

type IdleTimeoutMonitorProps = PropsWithChildren<{
  vaultState: VaultState | null;
  setVaultState: (maybeVaultState: VaultState | null) => void;
}>;

export const IdleTimeoutMonitor = ({
  children,
  vaultState,
  setVaultState,
}: IdleTimeoutMonitorProps) => {
  const lastInteractionRef = useRef(moment());

  const resetIdleTimeout = useCallback(() => {
    lastInteractionRef.current = moment();
    setSecondsRemainingBeforeSoftLock(IDLE_TIMEOUT.asSeconds());
  }, []);

  const [password, setPassword] = useState<string>("");
  const [isCheckingPassword, setIsCheckingPassword] = useState<boolean>(false);
  const [isPasswordWrong, setIsPasswordWrong] = useState<boolean>(false);

  const [secondsRemainingBeforeSoftLock, setSecondsRemainingBeforeSoftLock] =
    useState<number>(IDLE_TIMEOUT.asSeconds());

  useEffect(() => {
    const interval = setInterval(async () => {
      const secondsSinceLastInteraction = moment().diff(
        lastInteractionRef.current,
        "seconds",
      );

      if (vaultState && !vaultState.isSoftLocked) {
        setSecondsRemainingBeforeSoftLock((prevSecondsRemaining) => {
          const newSecondsRemaining =
            IDLE_TIMEOUT.asSeconds() - secondsSinceLastInteraction;

          if (newSecondsRemaining < 0 && prevSecondsRemaining >= 0) {
            sendDesktopNotification({
              body: `🔐 Sentinel has been passphrase-protected after ${IDLE_TIMEOUT.asMinutes().toFixed()} minutes of inactivity.`,
            });
            softLockVault().then(setVaultState);
          } else if (
            newSecondsRemaining < SECONDS_BEFORE_SOFTLOCK_TO_SHOW_WARNING &&
            prevSecondsRemaining >= SECONDS_BEFORE_SOFTLOCK_TO_SHOW_WARNING
          ) {
            sendDesktopNotification({
              body: `⚠️ Sentinel will be passphrase-protected in ${IDLE_WARNING_DURATION.asMinutes().toFixed()} minutes, due to inactivity.`,
            });
          }

          return newSecondsRemaining;
        });
      } else if (
        !vaultState &&
        // only notify every 24 hours after last interaction
        // (effectively daily if they're not logged in, but Sentinel running)
        secondsSinceLastInteraction > ONE_DAY_IN_SECONDS
      ) {
        lastInteractionRef.current = moment(); // reset timer
        sendDesktopNotification({
          body: `💡 It's recommended to be logged into your vault at all times for optimal security and functionality.`,
        });
      }
    }, 1000); // once per second
    return () => clearInterval(interval);
  }, [vaultState]);

  const minsBeforeSoftLock = Math.floor(secondsRemainingBeforeSoftLock / 60);
  const minsBeforeSoftLockStr =
    minsBeforeSoftLock > 0 && `${minsBeforeSoftLock}m`;
  const secsBeforeSoftLock = secondsRemainingBeforeSoftLock % 60;

  const dismissWarning = () => {
    lastInteractionRef.current = moment();
    setSecondsRemainingBeforeSoftLock(IDLE_TIMEOUT.asSeconds());
  };

  const verifyPassword = () => {
    setIsCheckingPassword(true);
    unlockSoftLockedVault(password).then((vaultState) => {
      if (vaultState?.isSoftLocked) {
        // vault still soft locked, passphrase must be wrong
        setIsPasswordWrong(true);
      } else if (vaultState) {
        resetIdleTimeout();
        setPassword("");
      }
      setVaultState(vaultState);
      setIsCheckingPassword(false);
    });
  };

  if (vaultState?.isSoftLocked) {
    return (
      <EuiOverlayMask>
        <div
          style={{
            textAlign: "center",
            userSelect: "none",
            WebkitUserSelect: "none",
          }}
        >
          <EuiTitle>
            <span>Please re-enter your vault passphrase</span>
          </EuiTitle>
          <EuiSpacer size="s" />
          <EuiText color="subdued">
            Sentinel is still checking for messages but following a period of
            inactivity you must re-enter your passphrase to view them.
          </EuiText>
          <EuiSpacer size="m" />
          <EuiText>
            Vault Path: <code>{vaultState.path}</code>
          </EuiText>
          <EuiSpacer size="m" />
          <EuiFieldPassword
            autoFocus
            value={password}
            onChange={(e) => {
              setPassword(e.target.value);
              setIsPasswordWrong(false);
            }}
            onKeyUp={(e) => e.key === "Enter" && verifyPassword()}
            type="dual"
            placeholder="Enter your passphrase"
            style={{ textAlign: "center" }}
            autoCapitalize="none"
            autoCorrect="off"
            spellCheck="false"
            isInvalid={isPasswordWrong}
            fullWidth
            disabled={isCheckingPassword}
          />
          {isPasswordWrong && (
            <EuiText size="s" color="darkred">
              Failed to unlock, please try again.
            </EuiText>
          )}
          <EuiSpacer size="m" />
          <EuiButton
            type="submit"
            disabled={password.length === 0 || isCheckingPassword}
            onClick={verifyPassword}
            isLoading={isCheckingPassword}
          >
            Verify passphrase
          </EuiButton>
        </div>
      </EuiOverlayMask>
    );
  } else if (
    secondsRemainingBeforeSoftLock < SECONDS_BEFORE_SOFTLOCK_TO_SHOW_WARNING
  ) {
    return (
      <EuiOverlayMask style="cursor: pointer">
        <EuiFocusTrap onClickOutside={dismissWarning}>
          {secondsRemainingBeforeSoftLock < 0 ? (
            <EuiLoadingSpinner size="xl" />
          ) : (
            <div
              style={{
                textAlign: "center",
                userSelect: "none",
                WebkitUserSelect: "none",
              }}
              onMouseOver={dismissWarning}
            >
              <EuiTitle>
                <span>Are you still there?</span>
              </EuiTitle>
              <EuiSpacer size="m" />
              <EuiText>
                {minsBeforeSoftLockStr} {Math.max(secsBeforeSoftLock, 0)}s
                before soft lock
              </EuiText>
            </div>
          )}
        </EuiFocusTrap>
      </EuiOverlayMask>
    );
  }

  return (
    <div
      onKeyUpCapture={resetIdleTimeout}
      onClickCapture={resetIdleTimeout}
      onFocusCapture={resetIdleTimeout}
    >
      {children}
    </div>
  );
};
