import * as _ from "radash";
import { create } from "zustand";
import { Message } from "../model/bindings/Message";

type MessageState = {
  messages: Message[];
  setMessages: (messages: Message[]) => void;
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
}));
