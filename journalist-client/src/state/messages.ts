import * as _ from "radash";
import { create } from "zustand";
import { Message } from "../model/bindings/Message";

type MessageState = {
  messages: Message[];
  setMessages: (messages: Message[]) => void;
  markAsRead: (id: bigint) => void;
};

export const useMessageStore = create<MessageState>((set) => ({
  messages: [],
  setMessages: (messages: Message[]) =>
    set((state) => {
      if (_.isEqual(state.messages, messages)) {
        return state;
      } else {
        return {
          messages,
        };
      }
    }),
  markAsRead: (id: bigint) =>
    set((state) => ({
      messages: state.messages.map((message: Message) =>
        message.type === "userToJournalistMessage" && message.id === id
          ? { ...message, read: true }
          : message,
      ),
    })),
}));
