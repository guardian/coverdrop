import type { CSSProperties } from "react";

type Sizes = Record<string, CSSProperties>;

const widths = {
  genericModalWidth: "600px",
};

export const sizes = {
  chatInput: {
    minHeight: "45px",
  },
  chatMessage: {
    minWidth: "100px",
  },
  chatsSideBar: {
    height: "100vh",
    minWidth: "325px",
  },
  addTrustAnchorModal: {
    width: widths.genericModalWidth,
  },
  coverMessageBurstModal: {
    width: widths.genericModalWidth,
  },
  muteModal: {
    width: widths.genericModalWidth,
  },
  copyToClipboardModal: {
    width: widths.genericModalWidth,
  },
  trustedKeyDigestModal: {
    width: widths.genericModalWidth,
  },
} as const satisfies Sizes;
