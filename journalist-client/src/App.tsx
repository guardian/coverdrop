import { ReactElement, useEffect, useState } from "react";

import {
  EuiPageTemplate,
  EuiPageTemplateProps,
  EuiGlobalToastList,
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
import { BackupModal } from "./components/BackupModal.tsx";

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

  useEffect(() => {
    applyPalette(colorMode.toLowerCase() as ColorMode);
  }, [colorMode]);

  const [vaultState, setVaultState] = useState<VaultState | null>(null);
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

  const [customToasts, setCustomToasts] = useState<Toast[]>([]);
  const addCustomToast = (toast: Toast) => {
    setCustomToasts((prev) => [toast, ...prev]);
  };
  const removeCustomToast = (toastId: string) =>
    setCustomToasts((prev) => prev.filter((_) => _.id !== toastId));

  const errorToasts = errorsState.errors.map((e) => ({
    id: e.id,
    title: "Error",
    color: "danger" as const,
    iconType: "warning",
    text: <p>{e.message}</p>,
  }));

  const toasts = [...customToasts, ...errorToasts];

  const messageStore = useMessageStore();
  const userStore = useUserStore();

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
    const intervalId = setInterval(() => {
      fetchUsersAndChats();
    }, 5000);

    return () => clearInterval(intervalId);
  }, [vaultState]);

  const userInfo = userStore.getUserInfo();

  useEffect(() => {
    getVaultState().then((s) => {
      setVaultState(s);
    });
  }, []);

  const markChatAsUnread = (replyKey: string) => {
    const messagesFromUser = messageStore.messages.filter(
      (msg) =>
        msg.userPk === replyKey && msg.type === "userToJournalistMessage",
    );
    if (replyKey === currentUserReplyKey) {
      setCurrentUserReplyKey(null); // must clear before markAsUnread, so it doesn't get immediately marked as read elsewhere
    }
    if (messagesFromUser.length > 0) {
      markAsUnread(messagesFromUser[messagesFromUser.length - 1].id);
    } else {
      useErrorStore
        .getState()
        .addWarning("Cannot mark chat as unread, no messages found from user.");
    }
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
          <EuiPageTemplate.Sidebar
            style={{
              position: "sticky",
              top: "0",
              height: "100vh",
              overflowY: "auto",
              padding: size.s,
            }}
            minWidth={325}
          >
            <ChatsSideBar
              journalistId={vaultState.id}
              journalistStatus={journalistProfile?.status}
              currentUserReplyKey={currentUserReplyKey}
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
            />
          </EuiPageTemplate.Sidebar>
          {currentUserReplyKey ? (
            <Chat
              messages={messageStore.messages}
              userReplyKey={currentUserReplyKey}
              userAutogeneratedName={userInfo[currentUserReplyKey].displayName}
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

          <BackupModal
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
  );
};

export default App;
