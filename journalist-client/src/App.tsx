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
import { getVaultState, softLockVault } from "./commands/vaults";
import { VaultState } from "./model/bindings/VaultState";
import { UserChat } from "./components/UserChat";
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
import { CreateOrEditGroupModal } from "./components/CreateOrEditGroupModal.tsx";
import { GroupChat } from "./components/GroupChat.tsx";
import { GroupWithComputed, useGroups } from "./state/groups.ts";

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

  const { maybeGroups, groupsUnreadCount } = useGroups(
    vaultState?.sentinelId || null,
  );

  const [isImportantStuffInProgress, setIsImportantStuffInProgress] =
    useState(false);

  const [maybeHungAt, setMaybeHungAt] = useState<Date | null>(null);

  // null = never backed up, undefined = not yet loaded
  const [lastBackupTime, setLastBackupTime] = useState<Date | null | undefined>(
    undefined,
  );

  useTrayIcon({
    maybeOpenVaultId: vaultState?.journalistId,
    isSoftLocked: vaultState?.isSoftLocked ?? false,
    isImportantStuffInProgress,
    isHung: !!maybeHungAt,
    sourceMessageUnreadCount: messageStore.messages.filter(
      (msg) =>
        msg.type === "userToJournalistMessage" &&
        !msg.read &&
        userStore.users.find((user) => user.userPk === msg.userPk)?.status ===
          "ACTIVE",
    ).length,
    groupsUnreadCount,
    lastBackupTime,
    onLockNow: () => {
      softLockVault().then(setVaultState);
    },
  });

  const [maybeCurrentUserKeyOrGroup, setMaybeCurrentUserKeyOrGroup] = useState<
    string | GroupWithComputed | null
  >(null);

  const maybeCurrentUserReplyKey =
    typeof maybeCurrentUserKeyOrGroup === "string"
      ? maybeCurrentUserKeyOrGroup
      : null;

  const maybeSelectedGroup =
    maybeCurrentUserKeyOrGroup && typeof maybeCurrentUserKeyOrGroup !== "string"
      ? maybeCurrentUserKeyOrGroup
      : null;

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
          `Journalist profile for ${journalistId} not found. It may not yet have been posted to the API.`,
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
      fetchPublicInfoAndSetJournalistProfile(vaultState.journalistId);
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

  const [shouldShowCreateGroupModal, setShouldShowCreateGroupModal] =
    useState(false);

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
  const refreshUsersAndChats = async () => {
    const [users, messages] = await Promise.all([getUsers(), getChats()]);
    userStore.setUsers(users);
    messageStore.setMessages(messages);
  };
  useEffect(() => {
    // return early if there isn't an open vault
    if (vaultState === null) {
      return;
    }

    refreshUsersAndChats();
    const intervalId = setInterval(refreshUsersAndChats, 5000);

    return () => clearInterval(intervalId);
  }, [vaultState]);

  const userInfo = userStore.getUserInfo();

  useEffect(() => {
    getVaultState().then((s) => {
      setVaultState(s);
    });
  }, []);

  const markChatAsUnread = async (replyKey: string) => {
    if (replyKey === maybeCurrentUserReplyKey) {
      setMaybeCurrentUserKeyOrGroup(null); // must clear before markAsUnread, so it doesn't get immediately marked as read elsewhere
    }
    await markAsUnread(replyKey);
    getUsers().then(userStore.setUsers);
  };

  useEffect(() => {
    const unreadMessageHasArrivedForSelectedUser =
      maybeCurrentUserReplyKey &&
      messageStore.messages.some(
        (msg) =>
          msg.userPk === maybeCurrentUserReplyKey &&
          msg.type === "userToJournalistMessage" &&
          !msg.read,
      );
    if (unreadMessageHasArrivedForSelectedUser) {
      setMaybeCurrentUserKeyOrGroup(null);
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
                    "You cannot dismiss this message. Please contact the digital investigations team.",
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
                    If this message persists for more than a few seconds, please
                    contact the digital investigations team for help.
                  </p>
                  Ideally leave Sentinel running at this screen, but if needs be
                  you can restart Sentinel to try to resolve the issue. (Please
                  take a screenshot first to share with us.)
                </EuiModalBody>
              </EuiModal>
            )}
            <EuiPageTemplate.Sidebar
              style={{
                height: sizes.chatsSideBar.height,
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
                journalistId={vaultState.journalistId}
                sentinelId={vaultState.sentinelId}
                journalistStatus={journalistProfile?.status}
                maybeCurrentUserReplyKey={maybeCurrentUserReplyKey}
                maybeSelectedGroup={maybeSelectedGroup}
                lastBackupTime={lastBackupTime}
                setChat={setMaybeCurrentUserKeyOrGroup}
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
                openCreateGroupModal={() => setShouldShowCreateGroupModal(true)}
                addCustomToast={addCustomToast}
                removeCustomToast={removeCustomToast}
                maybeGroups={maybeGroups}
                groupsUnreadCount={groupsUnreadCount}
              />
            </EuiPageTemplate.Sidebar>
            {maybeCurrentUserReplyKey && (
              <UserChat
                messages={messageStore.messages}
                userReplyKey={maybeCurrentUserReplyKey}
                userAutogeneratedName={
                  userInfo[maybeCurrentUserReplyKey].displayName
                }
                currentUserStatus={userInfo[maybeCurrentUserReplyKey].status}
                userAlias={userInfo[maybeCurrentUserReplyKey].alias}
                userDescription={userInfo[maybeCurrentUserReplyKey].description}
                markAsUnread={() => markChatAsUnread(maybeCurrentUserReplyKey)}
                showEditModal={() =>
                  setMaybeEditModalForReplyKey(maybeCurrentUserReplyKey)
                }
                showMuteModal={() =>
                  setMaybeMuteModalForReplyKey(maybeCurrentUserReplyKey)
                }
                showCopyToClipboardModal={() =>
                  setMaybeCopyToClipboardModalForReplyKey(
                    maybeCurrentUserReplyKey,
                  )
                }
              />
            )}
            {vaultState.sentinelId && maybeSelectedGroup && (
              <GroupChat
                key={maybeSelectedGroup.id}
                group={maybeSelectedGroup}
                sentinelId={vaultState.sentinelId}
                close={() => setMaybeCurrentUserKeyOrGroup(null)}
              />
            )}
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
              fetchUsersAndChats={refreshUsersAndChats}
            />

            <EditUserModal
              maybeReplyKey={maybeEditModalForReplyKey}
              closeModal={() => setMaybeEditModalForReplyKey(null)}
              fetchUsersAndChats={refreshUsersAndChats}
            />

            <CopyToClipboardModal
              maybeReplyKey={maybeCopyToClipboardModalForReplyKey}
              closeModal={() => setMaybeCopyToClipboardModalForReplyKey(null)}
              vaultId={vaultState.journalistId}
            />

            <ManualBackupModal
              isOpen={isBackupModalOpen}
              vaultPath={vaultState.path}
              setIsBackupModalOpen={setIsBackupModalOpen}
              addCustomToast={addCustomToast}
              removeCustomToast={removeCustomToast}
            />

            {vaultState.sentinelId && shouldShowCreateGroupModal && (
              <CreateOrEditGroupModal
                close={() => setShouldShowCreateGroupModal(false)}
                sentinelId={vaultState.sentinelId}
              />
            )}
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
