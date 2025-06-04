import { Profiles } from "../model/bindings/Profiles";
import { invokeWithErrorMessage } from "./invokeWithErrorMessage";

export const getProfiles = (): Promise<Profiles> => {
  return invokeWithErrorMessage("get_profiles");
};
