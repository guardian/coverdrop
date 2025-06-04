export type ColorMode = "light" | "dark";

type Palette = Record<`--${string}`, Record<ColorMode, string>>;

type PaletteKey = keyof typeof PALETTE extends `--${infer CSSVarName}`
  ? CSSVarName
  : never;

/**
 * Wraps a variable name in the CSS `var()` function
 * The type system ensures that CSS variable names exists in the {@link PALETTE} object
 *
 * @example
 * palette("chat-header-background") returns the string `var(--chat-header-background)`
 */
export const palette = (cssVarName: PaletteKey): `var(--${string})` =>
  `var(--${cssVarName})`;

/*
 * Runs when the App is started (or whenever the color mode is changed) and
 * sets the color palette at the root element.
 */
export const applyPalette = (colorMode: ColorMode) => {
  const root = document.documentElement;

  Object.entries(PALETTE).forEach(([name, value]) => {
    root.style.setProperty(name, value[colorMode]);
  });
};

/**
 * The source of truth for colors used in the Journalist Client.
 */
const PALETTE = {
  "--chat-header-background": {
    light: "#FFFFFF",
    dark: "#FFFFFF",
  },
  "--chat-header-border-color": {
    light: "#D3DAE6",
    dark: "#D3DAE6",
  },
  "--chat-sidebar-journalist-name-border-color": {
    light: "#D3DAE6",
    dark: "#D3DAE6",
  },
  "--chat-sidebar-message-preview-color": {
    light: "#4A4F55",
    dark: "#4A4F55",
  },
  "--chat-sidebar-selected-chat-background": {
    light: "#DDDDDD",
    dark: "#DDDDDD",
  },
  "--chat-sidebar-unread-message-dot-background": {
    light: "#0066CC",
    dark: "#0066CC",
  },
  "--chat-sidebar-unread-message-time-color": {
    light: "#0066CC",
    dark: "#0066CC",
  },
  "--chat-sidebar-unselected-chat-background": {
    light: "transparent",
    dark: "transparent",
  },
  "--journalist-to-user-message-background": {
    light: "#1133FF",
    dark: "#1133FF",
  },
  "--journalist-to-user-message-color": {
    light: "#EEEEEE",
    dark: "#EEEEEE",
  },
  "--message-sending-form-background": {
    light: "#DDDDDD",
    dark: "#DDDDDD",
  },
  "--message-sending-filling-progress-bar-background": {
    light: "#1133FF",
    dark: "#1133FF",
  },
  "--message-sending-full-progress-bar-background": {
    light: "#FF4444",
    dark: "#FF4444",
  },
  "--message-sending-progress-bar-background": {
    light: "#EEEEEE",
    dark: "#EEEEEE",
  },
  "--message-status-color": {
    light: "#AAAAAA",
    dark: "#AAAAAA",
  },
  "--user-to-journalist-message-background": {
    light: "#EEEEEE",
    dark: "#EEEEEE",
  },
  "--user-to-journalist-message-color": {
    light: "#333333",
    dark: "#333333",
  },
} as const satisfies Palette;
