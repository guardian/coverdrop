// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { JournalistIdentity } from "./JournalistIdentity";
import type { JournalistStatus } from "./JournalistStatus";
import type { RecipientTag } from "./RecipientTag";

export type JournalistProfile = {
  id: JournalistIdentity;
  displayName: string;
  sortName: string;
  description: string;
  isDesk: boolean;
  tag: RecipientTag;
  status: JournalistStatus;
};
