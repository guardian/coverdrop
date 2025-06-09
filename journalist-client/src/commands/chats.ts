import { Message } from "../model/bindings/Message";
import { User } from "../model/bindings/User";
import { UserStatus } from "../model/bindings/UserStatus";
import { useMessageStore } from "../state/messages";
import { invokeWithErrorMessage } from "./invokeWithErrorMessage";

export const getUsers = (): Promise<User[]> => {
  return invokeWithErrorMessage("get_users");
};

export const getChats = (): Promise<Message[]> => {
  return invokeWithErrorMessage("get_chats");
};

export const submitMessage = (
  replyKey: string,
  message: string,
): Promise<void> => {
  return invokeWithErrorMessage("submit_message", {
    replyKey,
    message,
  });
};

export const checkMessageLength = (message: string): Promise<number> => {
  return invokeWithErrorMessage("check_message_length", { message });
};

export const markAsRead = async (messageId: bigint): Promise<void> => {
  await invokeWithErrorMessage("mark_as_read", { messageId });
  useMessageStore.getState().markAsRead(messageId);
};

export const updateUserStatus = async (
  replyKey: string,
  status: UserStatus,
): Promise<void> => {
  await invokeWithErrorMessage("update_user_status", { replyKey, status });
};

export const updateUserAliasAndDescription = async (
  replyKey: string,
  alias: string,
  description: string,
): Promise<void> => {
  await invokeWithErrorMessage("update_user_alias_and_description", {
    replyKey,
    alias,
    description,
  });
};

export const burstCoverMessages = (count: number): Promise<void> => {
  return invokeWithErrorMessage("burst_cover_messages", { count });
};
