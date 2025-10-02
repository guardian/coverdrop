import * as _ from "radash";
import { create } from "zustand";
import { UntrustedKeysAndJournalistProfiles } from "../model/bindings/UntrustedKeysAndJournalistProfiles";

type PublicInfoState = {
  publicInfo?: UntrustedKeysAndJournalistProfiles;
  setPublicInfo: (publicInfo: UntrustedKeysAndJournalistProfiles) => void;
  getPublicInfo: () => UntrustedKeysAndJournalistProfiles | undefined;
};

export const usePublicInfoStore = create<PublicInfoState>((set, get) => ({
  publicInfo: undefined,
  setPublicInfo: (publicInfo: UntrustedKeysAndJournalistProfiles) =>
    set((state) => {
      if (_.isEqual(state.publicInfo, publicInfo)) {
        return state;
      } else {
        return {
          publicInfo,
        };
      }
    }),
  getPublicInfo: () => get().publicInfo,
}));
