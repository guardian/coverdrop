import { EuiProvider, useEuiTheme } from "@elastic/eui";
import "../src/euiIconsWorkAround.ts";
import type { Preview } from "@storybook/react-vite";
import { useEffect } from "react";
import { applyPalette, ColorMode } from "../src/styles/palette.ts";

// Journalist and user messages have IDs of type BigInt,
// which Storybook does not know how to serialize
// This renders them as regular numbers in the Storybook UI args panel.
declare global {
  interface BigInt {
    toJSON(): number;
  }
}
BigInt.prototype.toJSON = function () {
  return Number(this);
};

const EuiDecorator = ({ children }: { children: React.ReactNode }) => {
  const { colorMode } = useEuiTheme();
  useEffect(() => {
    applyPalette(colorMode.toLowerCase() as ColorMode);
  }, [colorMode]);

  return <>{children}</>;
};

const preview: Preview = {
  decorators: [
    (Story, context) => {
      const mode = context.globals.colorMode ?? "light";
      return (
        <EuiProvider colorMode={mode}>
          <EuiDecorator>
            <Story />
          </EuiDecorator>
        </EuiProvider>
      );
    },
  ],
  globalTypes: {
    colorMode: {
      description: "EUI Color Mode",
      defaultValue: "light",
      toolbar: {
        title: "Color Mode",
        icon: "contrast",
        items: [
          { value: "light", title: "Light mode", icon: "sun" },
          { value: "dark", title: "Dark mode", icon: "moon" },
        ],
        dynamicTitle: true,
      },
    },
  },
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
};

export default preview;
