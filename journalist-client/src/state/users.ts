import * as _ from "radash";
import { create } from "zustand";
import { User } from "../model/bindings/User";

type UserInfo = Omit<User, "type" | "userPk">;

type UserState = {
  users: User[];
  setUsers: (users: User[]) => void;
  getUserInfo: () => Record<string, UserInfo>;
};

export const useUserStore = create<UserState>((set, get) => ({
  users: [],
  setUsers: (users: User[]) =>
    set((state) => {
      if (_.isEqual(state.users, users)) {
        return state;
      } else {
        return {
          users,
        };
      }
    }),
  getUserInfo: () =>
    get().users.reduce<Record<string, UserInfo>>((acc, user) => {
      acc[user.userPk] = {
        alias: user.alias,
        displayName: user.displayName,
        description: user.description,
        status: user.status,
      };
      return acc;
    }, {}),
}));
