import type { SentinelProfile } from "../model/bindings/SentinelProfile.ts";
import { usePublicInfoStore } from "../state/publicInfo.ts";
import { Fragment, useMemo } from "react";
import { ToolTip } from "./ToolTip.tsx";

type SentinelProfileDisplayProps =
  | SentinelProfile
  | {
      id: string; // using it this way will look up the display name etc.
    };

export const SentinelProfileDisplay = (
  fullProfileOrJustId: SentinelProfileDisplayProps,
) => {
  const maybeSentinelProfiles =
    usePublicInfoStore().publicInfo?.sentinel_profiles;
  const { id, display_name } = useMemo(() => {
    if ("display_name" in fullProfileOrJustId) {
      return fullProfileOrJustId;
    }
    // TODO consider converting your own id to 'You'
    return (
      maybeSentinelProfiles?.find(
        (profile) => profile.id === fullProfileOrJustId.id,
      ) ?? {
        id: fullProfileOrJustId.id,
        display_name: fullProfileOrJustId.id, // use the id for display_name if we can't find the profile
      }
    );
  }, [fullProfileOrJustId]);

  return (
    <ToolTip position="top" content={`ID: ${id}`}>
      <span style={{ cursor: "help" }}>{display_name}</span>
    </ToolTip>
  );
};

export const SentinelProfilesDisplay = ({ ids }: { ids: string[] }) =>
  ids.map((id, index) => (
    <Fragment key={id}>
      <SentinelProfileDisplay id={id} />
      {index < ids.length - 2 && ", "}
      {index === ids.length - 2 && " & "}
    </Fragment>
  ));
