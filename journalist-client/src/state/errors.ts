import { create } from "zustand";

type Error = {
  id: string;
  message: string;
  title: string;
  color: "danger" | "warning";
};

type ErrorState = {
  nextErrorId: number;
  errors: Error[];
  addError: (message: string) => void;
  addWarning: (message: string) => void;
  removeError: (id: string) => void;
};

export const useErrorStore = create<ErrorState>((set) => ({
  nextErrorId: 0,
  errors: [],

  addWarning: (message: string) =>
    set((state) => {
      const warning = {
        id: `warning-${state.nextErrorId}`,
        title: "Warning",
        color: "warning" as const,
        message,
      };

      return {
        nextErrorId: state.nextErrorId + 1,
        errors: [warning, ...state.errors],
      };
    }),

  addError: (message: string) =>
    set((state) => {
      const error = {
        id: `error-${state.nextErrorId}`,
        color: "danger" as const,
        message,
        title: "Error",
      };

      return {
        nextErrorId: state.nextErrorId + 1,
        errors: [error, ...state.errors],
      };
    }),

  removeError: (id: string) =>
    set((state) => {
      const errors = state.errors.filter((e) => e.id !== id);
      return { errors };
    }),
}));
