import { useEffect, useState } from "react";
import { TrayIcon, TrayIconEvent } from "@tauri-apps/api/tray";
import { defaultWindowIcon, getName } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMessageStore } from "../state/messages.ts";
import { Image } from "@tauri-apps/api/image";
import { useUserStore } from "../state/users.ts";

interface UseTrayIconProps {
  isVaultOpen: boolean;
  isImportantStuffInProgress: boolean;
}

export const useTrayIcon = ({
  isVaultOpen,
  isImportantStuffInProgress,
}: UseTrayIconProps) => {
  const [maybeTrayPromise, setMaybeTrayPromise] = useState<Promise<TrayIcon>>();

  const [hasUnreadMessages, setHasUnreadMessages] = useState(false);
  const { messages } = useMessageStore();
  const { users } = useUserStore();
  useEffect(() => {
    setHasUnreadMessages(
      messages.some(
        (msg) =>
          msg.type === "userToJournalistMessage" &&
          !msg.read &&
          users.find((user) => user.userPk === msg.userPk)?.status === "ACTIVE",
      ),
    );
  }, [messages, users]);

  useEffect(() => {
    const trayPromise = (async () => {
      // for some reason the handler needs to be declared as a variable, rather than inline in the function call
      const handleTrayEvent = (event: TrayIconEvent) => {
        if (
          event.type === "Click" &&
          event.button === "Left" &&
          event.buttonState === "Up"
        ) {
          getCurrentWindow()
            .isFocused()
            .then((isFocused) => {
              if (isFocused) {
                getCurrentWindow().hide();
              } else {
                getCurrentWindow().unminimize();
                getCurrentWindow().show();
                getCurrentWindow().setFocus();
              }
            });
        }
      };
      return TrayIcon.new({
        id: "journalist-client-tray-icon",
        tooltip: await getName(),
        action: handleTrayEvent,
      });
    })();
    setMaybeTrayPromise(trayPromise);
    return () => {
      trayPromise.then((tray) => tray.close());
    };
  }, []);

  useEffect(() => {
    if (!maybeTrayPromise) {
      return;
    }
    const maybeSpinningIconIntervalPromise = Promise.all([
      maybeTrayPromise,
      defaultWindowIcon(),
    ]).then(async ([tray, maybeStartingIcon]) => {
      if (!maybeStartingIcon) {
        console.error("No default window icon found, cannot set tray icon");
        return;
      }
      const { width, height } = await maybeStartingIcon.size();
      const canvas = document.createElement("canvas");
      canvas.width = width;
      canvas.height = height;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      const startingImageData = new ImageData(
        new Uint8ClampedArray(await maybeStartingIcon.rgba()),
        width,
        height,
      );
      ctx.putImageData(startingImageData, 0, 0);

      if (hasUnreadMessages) {
        // place a blue circle in the top right corner (same symbol as in Sentinel itself, to denote unread)
        ctx.beginPath();
        ctx.arc(width * 0.75, height * 0.25, width / 4, 0, 2 * Math.PI);
        ctx.fillStyle = "#0066CC";
        ctx.fill();
      }

      const afterImageData = ctx.getImageData(0, 0, width, height);

      if (!isVaultOpen) {
        // desaturate the image to greyscale when the vault is closed
        const pixels = afterImageData.data;
        for (let i = 0; i < pixels.length; i += 4) {
          const lightness = (pixels[i] + pixels[i + 1] + pixels[i + 2]) / 3;
          pixels[i] = lightness;
          pixels[i + 1] = lightness;
          pixels[i + 2] = lightness;
        }
      }
      const newIcon = await Image.new(
        afterImageData.data.buffer,
        width,
        height,
      );
      await tray.setIcon(newIcon);
      await getCurrentWindow().setIcon(newIcon);

      if (isImportantStuffInProgress) {
        // start spinning the icon round once per second
        let counter = 0;
        return setInterval(async () => {
          counter++;
          const finalCanvas = document.createElement("canvas");
          finalCanvas.width = width;
          finalCanvas.height = height;
          const finalCtx = finalCanvas.getContext("2d");
          if (!finalCtx) return;
          finalCtx.translate(width / 2, height / 2);
          finalCtx.rotate((Math.PI / 180) * (counter * 12)); // 12 degrees per frame
          finalCtx.translate(-width / 2, -height / 2);
          finalCtx.drawImage(canvas, 0, 0);
          const finalImageData = finalCtx.getImageData(0, 0, width, height);

          const newIcon = await Image.new(
            finalImageData.data.buffer,
            width,
            height,
          );
          await tray.setIcon(newIcon);
          await getCurrentWindow().setIcon(newIcon);
        }, 33); // roughly 30fps
      }
    });
    return () => {
      maybeSpinningIconIntervalPromise.then((maybeSpinnerInterval) =>
        clearInterval(maybeSpinnerInterval),
      );
    };
  }, [
    maybeTrayPromise,
    isVaultOpen,
    hasUnreadMessages,
    isImportantStuffInProgress,
  ]);
};
