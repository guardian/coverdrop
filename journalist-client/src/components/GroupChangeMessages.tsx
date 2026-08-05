import { PropsWithChildren, ReactNode } from "react";
import { EuiFlexGroup, EuiFlexItem, useEuiTheme } from "@elastic/eui";
import {
  SentinelProfileDisplay,
  SentinelProfilesDisplay,
} from "./SentinelProfileDisplay.tsx";
import { GroupMessage } from "../model/bindings/GroupMessage.ts";
import { palette } from "../styles/palette.ts";
import { ToolTip } from "./ToolTip.tsx";

interface GroupChangeMessageProps extends PropsWithChildren {
  timestamp: string;
  tooltip?: ReactNode;
}
const GroupChangeMessage = ({
  children,
  timestamp,
  tooltip,
}: GroupChangeMessageProps) => {
  const { euiTheme } = useEuiTheme();
  const { size } = euiTheme;
  return (
    <div style={{ color: "grey", textAlign: "center" }}>
      <EuiFlexGroup alignItems="center" gutterSize="s">
        <EuiFlexItem grow>
          <hr />
        </EuiFlexItem>
        <ToolTip position="top" content={tooltip}>
          <div style={{ cursor: "help" }}>{children}</div>
        </ToolTip>
        <EuiFlexItem grow>
          <hr />
        </EuiFlexItem>
      </EuiFlexGroup>
      <div
        style={{
          fontSize: size.m,
          color: palette("message-status-color"),
          marginTop: size.xs,
        }}
      >
        {timestamp}
      </div>
    </div>
  );
};

export const GroupMembersAdded = ({
  timestamp,
  idsAdded,
  addedById,
}: {
  timestamp: string;
  idsAdded: string[];
  addedById: string;
}) => (
  <GroupChangeMessage timestamp={timestamp}>
    <SentinelProfileDisplay id={addedById} /> added{" "}
    <strong>
      <SentinelProfilesDisplay ids={idsAdded} />
    </strong>
  </GroupChangeMessage>
);
export const GroupMembersRemoved = ({
  timestamp,
  idsRemoved,
  removedById,
}: {
  timestamp: string;
  idsRemoved: string[];
  removedById: string;
}) => (
  <GroupChangeMessage timestamp={timestamp}>
    <SentinelProfileDisplay id={removedById} /> removed{" "}
    <strong>
      <SentinelProfilesDisplay ids={idsRemoved} />
    </strong>
  </GroupChangeMessage>
);

export const GroupMemberLeft = ({
  timestamp,
  idWhoLeft,
}: {
  timestamp: string;
  idWhoLeft: string;
}) => (
  <GroupChangeMessage timestamp={timestamp}>
    <strong>
      <SentinelProfileDisplay id={idWhoLeft} />
    </strong>{" "}
    left the group
  </GroupChangeMessage>
);

const propertyChangeMessageMapping = {
  GroupNameChanged: {
    whatChanged: "name",
    groupInfoPropertyName: "display_name",
  } as const,
  GroupDescriptionChanged: {
    whatChanged: "description",
    groupInfoPropertyName: "description",
  } as const,
};
export const GroupPropertyChanged = ({
  timestamp,
  changedBy,
  changeMessageType,
  historyUpThisPoint,
  changedTo,
}: {
  timestamp: string;
  changedBy: string;
  changeMessageType: keyof typeof propertyChangeMessageMapping;
  historyUpThisPoint: GroupMessage[];
  changedTo: string;
}) => {
  const whatChanged =
    propertyChangeMessageMapping[changeMessageType].whatChanged;
  const changedFrom = historyUpThisPoint.reduce(
    (acc: string | undefined, { content }) => {
      if (changeMessageType in content) {
        // @ts-expect-error - given the if check above this check should be fine
        return content[changeMessageType];
      }
      if ("GroupInfo" in content) {
        // FIXME we need to send this even when we're the group creator, if we want the before/after
        return content.GroupInfo[
          propertyChangeMessageMapping[changeMessageType].groupInfoPropertyName
        ];
      }
      return acc;
    },
    undefined,
  );
  return (
    <GroupChangeMessage
      timestamp={timestamp}
      tooltip={
        changedFrom && (
          <>
            <strong>Before</strong>
            <br />
            {changedFrom}
            <br />
            <br />
            <strong>After</strong>
            <br />
            {changedTo}
          </>
        )
      }
    >
      <SentinelProfileDisplay id={changedBy} /> changed the group {whatChanged}
    </GroupChangeMessage>
  );
};
