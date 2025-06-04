import {
  EuiModal,
  EuiModalHeader,
  EuiModalHeaderTitle,
  EuiModalBody,
  EuiBasicTable,
  EuiCode,
} from "@elastic/eui";
import { useEffect, useState } from "react";
import { getTrustAnchorDigests } from "../commands/admin";
import { TrustedOrganizationPublicKeyAndDigest } from "../model/bindings/TrustedOrganizationPublicKeyAndDigest";
import { sizes } from "../styles/sizes";

export const TrustedKeyDigestsModal = (props: { closeModal: () => void }) => {
  const [digests, setDigests] = useState<
    TrustedOrganizationPublicKeyAndDigest[]
  >([]);

  useEffect(() => {
    getTrustAnchorDigests().then((orgPkDigests) => {
      setDigests(orgPkDigests);
    });
  }, []);

  const columns = [
    {
      field: "pkHex",
      name: "Organization Public Key Hex",
      render: (pkHex: string) => <EuiCode>{pkHex}</EuiCode>,
    },
    {
      field: "digest",
      name: "Digest",
      render: (digest: string) => <EuiCode>{digest}</EuiCode>,
    },
  ];

  return (
    <EuiModal
      style={{ width: sizes.trustedKeyDigestModal.width }}
      onClose={props.closeModal}
    >
      <EuiModalHeader>
        <EuiModalHeaderTitle>Trust Anchor Digests</EuiModalHeaderTitle>
      </EuiModalHeader>
      <EuiModalBody>
        <EuiBasicTable items={digests} columns={columns} />
      </EuiModalBody>
    </EuiModal>
  );
};
