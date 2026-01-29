import { ReactElement, useEffect, useState } from "react";

import {
  EuiGlobalToastList,
  EuiModal,
  EuiModalBody,
  EuiModalHeader,
  EuiModalHeaderTitle,
  EuiPageTemplate,
  EuiPageTemplateProps,
  useEuiTheme,
} from "@elastic/eui";
import { ChatsSideBar } from "./components/ChatsSideBar";
import { OpenVault } from "./views/OpenVault";
import { getVaultState } from "./commands/vaults";
import { VaultState } from "./model/bindings/VaultState";
import { Chat } from "./components/Chat";
import { Toast } from "@elastic/eui/src/components/toast/global_toast_list";
import { useErrorStore } from "./state/errors";
import { useMessageStore } from "./state/messages";
import { applyPalette, ColorMode } from "./styles/palette";
import { useUserStore } from "./state/users";
import { getChats, getUsers, markAsUnread } from "./commands/chats";
import { MuteToggleModal } from "./components/MuteToggleModal.tsx";
import { EditUserModal } from "./components/EditUserModal.tsx";
import { CopyToClipboardModal } from "./components/CopyToClipboardModal.tsx";
import { JournalistStatus } from "./model/bindings/JournalistStatus.ts";
import { ToggleJournalistStatusModal } from "./components/ToggleJournalistStatusModal.tsx";
import { getPublicInfo } from "./commands/admin.ts";
import { JournalistProfile } from "./model/bindings/JournalistProfile.ts";
import { JournalistIdentity } from "./model/bindings/JournalistIdentity.ts";
import { IdleTimeoutMonitor } from "./components/IdleTimeoutMonitor.tsx";
import { ManualBackupModal } from "./components/ManualBackupModal.tsx";
import { sizes } from "./styles/sizes.ts";
import { useTrayIcon } from "./hooks/useTrayIcon.ts";
import { BackgroundTaskTrackerWithLoadingBarIfApplicable } from "./components/BackgroundTaskTrackerWithLoadingBarIfApplicable.tsx";
import { usePublicInfoStore } from "./state/publicInfo.ts";
import { listen } from "@tauri-apps/api/event";
import { AlertPayload } from "./model/bindings/AlertPayload.ts";
import { getBackupHistory } from "./commands/backups.ts";

const App = ({
  panelled,
  bottomBorder = true,
  offset,
  grow,
}: {
  content?: ReactElement;
  sidebar?: ReactElement;
  panelled?: EuiPageTemplateProps["panelled"];
  bottomBorder?: EuiPageTemplateProps["bottomBorder"];
  // For fullscreen only
  offset?: EuiPageTemplateProps["offset"];
  grow?: EuiPageTemplateProps["grow"];
}) => {
  const {
    colorMode,
    euiTheme: { size },
  } = useEuiTheme();

  const messageStore = useMessageStore();
  const userStore = useUserStore();
  const publicInfoStore = usePublicInfoStore();

  useEffect(() => {
    applyPalette(colorMode.toLowerCase() as ColorMode);
  }, [colorMode]);

  const [vaultState, setVaultState] = useState<VaultState | null>(null);

  const [isImportantStuffInProgress, setIsImportantStuffInProgress] =
    useState(false);

  const [maybeHungAt, setMaybeHungAt] = useState<Date | null>(null);

  // null = never backed up, undefined = not yet loaded
  const [lastBackupTime, setLastBackupTime] = useState<Date | null | undefined>(
    undefined,
  );

  useTrayIcon({
    maybeOpenVaultId: vaultState?.id,
    isImportantStuffInProgress,
    isHung: !!maybeHungAt,
  });

  const [currentUserReplyKey, setCurrentUserReplyKey] = useState<string | null>(
    null,
  );
  const [journalistProfile, setJournalistProfile] =
    useState<JournalistProfile | null>(null);

  const setJournalistStatus = (newStatus: JournalistStatus) => {
    setJournalistProfile((prev: JournalistProfile | null) => {
      if (prev === null) return null;
      return { ...prev, status: newStatus };
    });
  };

  // Find journalist profile from public info object
  const fetchPublicInfoAndSetJournalistProfile = async (
    journalistId: JournalistIdentity,
  ) => {
    try {
      const publicInfo = await getPublicInfo();
      if (publicInfo === null) {
        return;
      }

      publicInfoStore.setPublicInfo(publicInfo);

      const journalistProfile = publicInfo.journalist_profiles.find(
        (p) => p.id == journalistId,
      );
      if (!journalistProfile) {
        console.warn(
          `Journalist profile for ${journalistId} not found. It might not have been posted to the API yet.`,
        );
        return;
      }
      setJournalistProfile(journalistProfile);
    } catch (err) {
      console.error("Failed to fetch data:", err);
    }
  };

  // attempt to set the initial value of the journalist profile every second until it's populated
  useEffect(() => {
    if (!vaultState) {
      return;
    }
    // only fetch public info if it hasn't been initialised
    if (journalistProfile !== null) {
      return;
    }
    const intervalId = setInterval(() => {
      fetchPublicInfoAndSetJournalistProfile(vaultState.id);
    }, 1000);
    return () => clearInterval(intervalId);
  }, [vaultState, journalistProfile]);

  // poll for last backup time every minute
  const pollLastBackupTime = async () => {
    console.log("Fetching backup history to update last backup time");
    getBackupHistory().then((history) => {
      console.log("Fetched backup history:", history);
      setLastBackupTime(
        history.length > 0 ? new Date(history[0].timestamp) : null,
      );
    });
  };

  useEffect(() => {
    if (!vaultState) {
      return;
    }
    pollLastBackupTime(); // initial fetch
    const intervalId = setInterval(() => {
      pollLastBackupTime();
    }, 60_000);
    return () => clearInterval(intervalId);
  }, [vaultState]);

  const [maybeJournalistStatusForModal, setMaybeJournalistStatusForModal] =
    useState<JournalistStatus | null>(null);

  const [isBackupModalOpen, setIsBackupModalOpen] = useState(false);

  const [maybeEditModalForReplyKey, setMaybeEditModalForReplyKey] = useState<
    string | null
  >(null);
  const [maybeMuteModalForReplyKey, setMaybeMuteModalForReplyKey] = useState<
    string | null
  >(null);
  const [
    maybeCopyToClipboardModalForReplyKey,
    setMaybeCopyToClipboardModalForReplyKey,
  ] = useState<string | null>(null);

  const errorsState = useErrorStore();

  // listen for generic alerts from the backend
  useEffect(() => {
    const listener = listen<AlertPayload>("notification", (event) => {
      console.log("Event received from backend", event);
      if (event.payload.level === "WARNING") {
        errorsState.addWarning(event.payload.message);
      } else if (event.payload.level === "ERROR") {
        errorsState.addError(event.payload.message);
      }
    });
    return () => {
      listener.then((unlisten) => unlisten());
    };
  }, []);

  const [customToasts, setCustomToasts] = useState<Toast[]>([]);
  const addCustomToast = (toast: Toast) => {
    setCustomToasts((prev) => [toast, ...prev]);
  };
  const removeCustomToast = (toastId: string) =>
    setCustomToasts((prev) => prev.filter((_) => _.id !== toastId));

  const errorToasts = errorsState.errors.map((e) => ({
    id: e.id,
    title: e.title,
    color: e.color,
    iconType: "warning",
    text: <p>{e.message}</p>,
  }));

  const toasts = [...customToasts, ...errorToasts];

  // Fetch initial messages and users and set interval to refresh them
  // every 5 seconds
  const fetchUsersAndChats = async () => {
    const [users, messages] = await Promise.all([getUsers(), getChats()]);
    userStore.setUsers(users);
    messageStore.setMessages(messages);
  };
  useEffect(() => {
    // return early if there isn't an open vault
    if (vaultState === null) {
      return;
    }

    fetchUsersAndChats();
    const intervalId = setInterval(fetchUsersAndChats, 5000);

    return () => clearInterval(intervalId);
  }, [vaultState]);

  const userInfo = userStore.getUserInfo();

  useEffect(() => {
    getVaultState().then((s) => {
      setVaultState(s);
    });
  }, []);

  const markChatAsUnread = async (replyKey: string) => {
    if (replyKey === currentUserReplyKey) {
      setCurrentUserReplyKey(null); // must clear before markAsUnread, so it doesn't get immediately marked as read elsewhere
    }
    await markAsUnread(replyKey);
    getUsers().then(userStore.setUsers);
  };

  useEffect(() => {
    const unreadMessageHasArrivedForSelectedUser =
      currentUserReplyKey &&
      messageStore.messages.some(
        (msg) =>
          msg.userPk === currentUserReplyKey &&
          msg.type === "userToJournalistMessage" &&
          !msg.read,
      );
    if (unreadMessageHasArrivedForSelectedUser) {
      setCurrentUserReplyKey(null);
    }
  }, [messageStore.messages.length]);

  return (
    <IdleTimeoutMonitor vaultState={vaultState} setVaultState={setVaultState}>
      <div>
        {vaultState === null ? (
          <OpenVault setVaultState={setVaultState} />
        ) : (
          <EuiPageTemplate
            panelled={panelled}
            bottomBorder={bottomBorder}
            grow={grow}
            offset={offset}
          >
            {maybeHungAt && (
              <EuiModal
                onClose={() =>
                  alert(
                    "You cannot dismiss this message, please contact the digital investigations team.",
                  )
                }
              >
                <EuiModalHeader>
                  <EuiModalHeaderTitle>
                    Sentinel has stopped receiving/sending messages
                  </EuiModalHeaderTitle>
                </EuiModalHeader>
                <EuiModalBody>
                  <p>
                    <em>This was detected {maybeHungAt.toString()}</em>
                  </p>
                  <p>
                    Please contact digital investigations team ASAP, to help us
                    track down the root cause.
                  </p>
                  Ideally leave Sentinel running at this screen, but if needs be
                  you can restart Sentinel to try to resolve the issue (please
                  take a screenshot first).
                </EuiModalBody>
              </EuiModal>
            )}
            <EuiPageTemplate.Sidebar
              style={{
                position: "sticky",
                top: "0",
                height: sizes.chatsSideBar.height,
                overflowY: "auto",
                padding: size.s,
              }}
              minWidth={sizes.chatsSideBar.minWidth}
            >
              <BackgroundTaskTrackerWithLoadingBarIfApplicable
                isImportantStuffInProgress={isImportantStuffInProgress}
                setIsImportantStuffInProgress={setIsImportantStuffInProgress}
                maybeHungAt={maybeHungAt}
                setMaybeHungAt={setMaybeHungAt}
              />
              <ChatsSideBar
                journalistId={vaultState.id}
                journalistStatus={journalistProfile?.status}
                currentUserReplyKey={currentUserReplyKey}
                lastBackupTime={lastBackupTime}
                setChat={setCurrentUserReplyKey}
                markChatAsUnread={markChatAsUnread}
                setMaybeEditModalForReplyKey={setMaybeEditModalForReplyKey}
                setMaybeMuteModalForReplyKey={setMaybeMuteModalForReplyKey}
                setMaybeCopyToClipboardModalForReplyKey={
                  setMaybeCopyToClipboardModalForReplyKey
                }
                setMaybeJournalistStatusForModal={
                  setMaybeJournalistStatusForModal
                }
                openBackupModal={() => setIsBackupModalOpen(true)}
                addCustomToast={addCustomToast}
                removeCustomToast={removeCustomToast}
              />
            </EuiPageTemplate.Sidebar>
            {currentUserReplyKey ? (
              <Chat
                messages={messageStore.messages}
                userReplyKey={currentUserReplyKey}
                userAutogeneratedName={
                  userInfo[currentUserReplyKey].displayName
                }
                currentUserStatus={userInfo[currentUserReplyKey].status}
                userAlias={userInfo[currentUserReplyKey].alias}
                userDescription={userInfo[currentUserReplyKey].description}
                markAsUnread={() => markChatAsUnread(currentUserReplyKey)}
                showEditModal={() =>
                  setMaybeEditModalForReplyKey(currentUserReplyKey)
                }
                showMuteModal={() =>
                  setMaybeMuteModalForReplyKey(currentUserReplyKey)
                }
                showCopyToClipboardModal={() =>
                  setMaybeCopyToClipboardModalForReplyKey(currentUserReplyKey)
                }
              />
            ) : null}

            {journalistProfile && (
              <ToggleJournalistStatusModal
                journalistProfile={journalistProfile}
                newStatus={maybeJournalistStatusForModal}
                setJournalistStatus={setJournalistStatus}
                closeModal={() => setMaybeJournalistStatusForModal(null)}
              />
            )}

            <MuteToggleModal
              maybeReplyKey={maybeMuteModalForReplyKey}
              closeModal={() => setMaybeMuteModalForReplyKey(null)}
              fetchUsersAndChats={fetchUsersAndChats}
            />

            <EditUserModal
              maybeReplyKey={maybeEditModalForReplyKey}
              closeModal={() => setMaybeEditModalForReplyKey(null)}
              fetchUsersAndChats={fetchUsersAndChats}
            />

            <CopyToClipboardModal
              maybeReplyKey={maybeCopyToClipboardModalForReplyKey}
              closeModal={() => setMaybeCopyToClipboardModalForReplyKey(null)}
              vaultId={vaultState.id}
            />

            <ManualBackupModal
              isOpen={isBackupModalOpen}
              vaultPath={vaultState.path}
              setIsBackupModalOpen={setIsBackupModalOpen}
              addCustomToast={addCustomToast}
              removeCustomToast={removeCustomToast}
            />
          </EuiPageTemplate>
        )}

        <EuiGlobalToastList
          toasts={toasts}
          dismissToast={(t: Toast) => {
            errorsState.removeError(t.id);
            removeCustomToast(t.id);
          }}
          toastLifeTimeMs={60_000}
        />
      </div>
    </IdleTimeoutMonitor>
  );
};

export default App;
