import {
  EuiButtonEmpty,
  EuiModal,
  EuiModalBody,
  EuiModalFooter,
  EuiModalHeader,
  EuiModalHeaderTitle,
  EuiCheckbox,
  EuiFieldSearch,
  EuiButton,
  EuiFormRow,
  EuiFieldText,
  EuiFlexGrid,
} from "@elastic/eui";
import { usePublicInfoStore } from "../state/publicInfo.ts";
import { useMemo, useState } from "react";
import { SentinelProfileDisplay } from "./SentinelProfileDisplay.tsx";
import { createGroup, modifyGroup } from "../commands/groups.ts";
import { Group } from "../model/bindings/Group.ts";

interface CreateOrEditGroupModalProps {
  close: () => void;
  sentinelId: string;
  maybeExistingGroupToEdit?: Group;
}

export const CreateOrEditGroupModal = ({
  close,
  sentinelId,
  maybeExistingGroupToEdit,
}: CreateOrEditGroupModalProps) => {
  const publicInfo = usePublicInfoStore().getPublicInfo();

  const [contactSearch, setContactSearch] = useState("");

  const filteredProfiles = useMemo(
    () =>
      publicInfo?.sentinel_profiles?.filter(
        ({ display_name, id }) =>
          !contactSearch ||
          [id, display_name].some((_) =>
            _.toLowerCase().includes(contactSearch.trim().toLowerCase()),
          ),
      ),
    [contactSearch, publicInfo],
  );

  const [groupName, setGroupName] = useState(
    maybeExistingGroupToEdit?.display_name ?? "",
  );
  const [groupDescription, setGroupDescription] = useState(
    maybeExistingGroupToEdit?.description ?? "",
  );
  const [selectedContactIds, setSelectedContactIds] = useState<string[]>(
    maybeExistingGroupToEdit?.members ?? [sentinelId],
  );

  return (
    <EuiModal style={{ width: "90vw", height: "90vh" }} onClose={close}>
      <EuiModalHeader>
        <EuiModalHeaderTitle>
          {maybeExistingGroupToEdit ? "Edit" : "Create"} MLS Group
        </EuiModalHeaderTitle>
      </EuiModalHeader>
      <EuiModalBody style={{ width: "100%", maxWidth: "none" }}>
        <EuiFormRow label={"Group name"} fullWidth>
          <EuiFieldText
            fullWidth
            required
            value={groupName}
            onChange={(e) => setGroupName(e.target.value)}
            placeholder={"Enter group name..."}
            autoCapitalize="off"
            autoCorrect="off"
          />
        </EuiFormRow>
        <EuiFormRow label={"Group description"} fullWidth>
          <EuiFieldText
            fullWidth
            required={false}
            value={groupDescription}
            onChange={(e) => setGroupDescription(e.target.value)}
            placeholder={"(optional) Enter group description..."}
            autoCapitalize="off"
            autoCorrect="off"
          />
        </EuiFormRow>
        <EuiFormRow label="Contacts" fullWidth>
          <div>
            <EuiFieldSearch
              fullWidth
              onChange={(e) => setContactSearch(e.target.value)}
              value={contactSearch}
              placeholder={"Search contacts..."}
              autoCapitalize="off"
              autoCorrect="off"
            />
            <EuiFlexGrid
              gutterSize="s"
              columns={2}
              alignItems={"start"}
              style={{ maxHeight: "40vh", overflowY: "scroll", padding: "5px" }}
            >
              {filteredProfiles?.map(({ id, ...restOfProfile }) => (
                <EuiCheckbox
                  key={id}
                  id={id}
                  disabled={id === sentinelId}
                  onChange={(e) =>
                    setSelectedContactIds((prev) =>
                      e.target.checked
                        ? [...prev, id]
                        : prev.filter((_) => _ !== id),
                    )
                  }
                  checked={selectedContactIds.includes(id)}
                  label={<SentinelProfileDisplay id={id} {...restOfProfile} />}
                />
              ))}
            </EuiFlexGrid>
          </div>
        </EuiFormRow>
      </EuiModalBody>
      <EuiModalFooter>
        <EuiButtonEmpty onClick={close}>Cancel</EuiButtonEmpty>
        <EuiButton
          disabled={groupName.length === 0 || selectedContactIds.length < 2}
          onClick={async () => {
            if (maybeExistingGroupToEdit) {
              await modifyGroup(
                maybeExistingGroupToEdit.id,
                groupName,
                groupDescription,
                selectedContactIds,
              );
            } else {
              await createGroup(
                groupName,
                groupDescription,
                selectedContactIds.filter((id) => id !== sentinelId),
              );
            }
            close();
          }}
        >
          {maybeExistingGroupToEdit ? "Save this " : "Create "}group (
          {selectedContactIds.length} contact
          {selectedContactIds.length > 1 ? "s" : ""})
        </EuiButton>
      </EuiModalFooter>
    </EuiModal>
  );
};
