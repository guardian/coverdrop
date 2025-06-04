import * as _ from "radash";
import { create } from "zustand";

type DraftState = {
  drafts: Record<string, string>;
  setDraft: (userReplyKey: string, draft: string) => void;
  clearDraft: (userReplyKey: string) => void;
};

export const useDraftStore = create<DraftState>((set) => ({
  drafts: {},
  setDraft: (userReplyKey: string, draft: string) =>
    set((state) => ({
      drafts: {
        ...state.drafts,
        [userReplyKey]: draft,
      },
    })),
  clearDraft: (userReplyKey: string) =>
    set((state) => ({
      drafts: _.omit(state.drafts, [userReplyKey]),
    })),
}));
