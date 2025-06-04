import { invoke, InvokeArgs } from "@tauri-apps/api/core";
import { useErrorStore } from "../state/errors";

export const invokeWithErrorMessage = <T>(
  name: string,
  args?: InvokeArgs | undefined,
): Promise<T> => {
  return invoke<T>(name, args).catch((e) => {
    useErrorStore.getState().addError(e);
    throw e;
  });
};

export const invokeWithSilencedErrorMessage = <T>(
  name: string,
  args?: InvokeArgs | undefined,
): Promise<T | null> => {
  return invoke<T>(name, args).catch((e) => {
    console.error(e);
    return null;
  });
};
