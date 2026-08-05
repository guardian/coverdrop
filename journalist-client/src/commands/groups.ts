import { invokeWithErrorMessage } from "./invokeWithErrorMessage";
import { Group } from "../model/bindings/Group.ts";

export const createGroup = (
  name: string,
  description: string,
  memberIds: string[],
): Promise<void> => {
  return invokeWithErrorMessage("create_group", {
    displayName: name,
    description,
    groupMembers: memberIds,
  });
};

export const modifyGroup = (
  groupId: string,
  name: string,
  description: string,
  memberIds: string[],
): Promise<void> => {
  return invokeWithErrorMessage("modify_group", {
    groupId,
    newGroupName: name,
    newDescription: description,
    newGroupMembers: memberIds,
  });
};

export const getGroup = (groupId: string): Promise<Group> => {
  return invokeWithErrorMessage("get_group_with_messages", { groupId });
};

export const getGroups = (): Promise<Group[]> => {
  return invokeWithErrorMessage("get_groups_and_messages");
};

export const sendGroupMessage = (
  groupId: string,
  message: string,
): Promise<void> => {
  return invokeWithErrorMessage("send_group_message", {
    groupId,
    message,
  });
};

export const setIsTypingInGroup = (
  groupId: string,
  isTyping: boolean,
): Promise<void> => {
  return invokeWithErrorMessage("user_typing", {
    groupId,
    isTyping,
  });
};

export const updateGroupMessageReadStatus = (
  groupId: string,
  messages: string[],
  isRead: boolean,
): Promise<void> => {
  return invokeWithErrorMessage("update_group_messages_read_status", {
    groupId,
    messages,
    read: isRead,
  });
};
