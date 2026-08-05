import { useEffect, useState } from "react";
import { TrayIcon, TrayIconEvent } from "@tauri-apps/api/tray";
import { defaultWindowIcon, getName } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Image } from "@tauri-apps/api/image";
import { Menu, MenuItem, PredefinedMenuItem } from "@tauri-apps/api/menu";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { fullyExitApp } from "../commands/admin.ts";
import { useBackendEventListener } from "./useBackendEventListener.tsx";

interface UseTrayIconProps {
  maybeOpenVaultId: string | undefined;
  isSoftLocked: boolean;
  isImportantStuffInProgress: boolean;
  isHung: boolean;
  sourceMessageUnreadCount: number;
  groupsUnreadCount: number;
  lastBackupTime: Date | null | undefined;
  onLockNow: () => void;
}

export const useTrayIcon = ({
  maybeOpenVaultId,
  isSoftLocked,
  isImportantStuffInProgress,
  isHung,
  sourceMessageUnreadCount,
  groupsUnreadCount,
  lastBackupTime,
  onLockNow,
}: UseTrayIconProps) => {
  const [maybeTrayPromise, setMaybeTrayPromise] = useState<Promise<TrayIcon>>();
  const [isWindowVisible, setIsWindowVisible] = useState(true);

  const hasUnreadMessages =
    sourceMessageUnreadCount > 0 || groupsUnreadCount > 0;

  const backgroundTasks = [
    {
      name: "Sending messages",
      data: useBackendEventListener("outbound_queue_length"),
    },
    {
      name: "Processing dead drops",
      data: useBackendEventListener("dead_drops_remaining"),
    },
    {
      name: "Performing vault backup",
      data: useBackendEventListener("automated_backup"),
    },
  ];
  const activeBackgroundTaskNames = backgroundTasks
    .filter(({ data }) => data.remainingCount !== 0)
    .map(({ name }) => name);

  // Track window visibility changes
  useEffect(() => {
    const window = getCurrentWindow();
    const unlisten = window.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        setIsWindowVisible(true);
      }
    });
    const unlistenClose = window.listen("tauri://close-requested", () => {
      setIsWindowVisible(false);
    });
    return () => {
      unlisten.then((fn) => fn());
      unlistenClose.then((fn) => fn());
    };
  }, []);

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
                setIsWindowVisible(false);
              } else {
                getCurrentWindow().unminimize();
                getCurrentWindow().show();
                getCurrentWindow().setFocus();
                setIsWindowVisible(true);
              }
            });
        }
      };
      return TrayIcon.new({
        id: "journalist-client-tray-icon",
        tooltip: await getName(),
        action: handleTrayEvent,
        showMenuOnLeftClick: false,
      });
    })();
    setMaybeTrayPromise(trayPromise);
    return () => {
      trayPromise.then((tray) => tray.close());
    };
  }, []);

  // Build and update the right-click context menu
  useEffect(() => {
    if (!maybeTrayPromise) {
      return;
    }

    const buildMenu = async () => {
      const tray = await maybeTrayPromise;
      const window = getCurrentWindow();

      const items: (MenuItem | PredefinedMenuItem)[] = [];

      // --- Vault status ---
      if (maybeOpenVaultId) {
        items.push(
          await MenuItem.new({
            text: `Vault: ${maybeOpenVaultId}`,
            enabled: false,
          }),
        );
      } else {
        items.push(
          await MenuItem.new({ text: "Vault: not logged in", enabled: false }),
        );
      }

      // --- Active tasks (shown below vault when tasks are running) ---
      if (activeBackgroundTaskNames.length > 0) {
        for (const taskName of activeBackgroundTaskNames) {
          items.push(
            await MenuItem.new({ text: `⏳ ${taskName}...`, enabled: false }),
          );
        }
      }

      items.push(await PredefinedMenuItem.new({ item: "Separator" }));

      // --- Messages & backup info ---
      if (sourceMessageUnreadCount > 0) {
        items.push(
          await MenuItem.new({
            text: `${sourceMessageUnreadCount.toLocaleString()} unread source message${sourceMessageUnreadCount === 1 ? "" : "s"}`,
            enabled: false,
          }),
        );
      }

      if (groupsUnreadCount > 0) {
        items.push(
          await MenuItem.new({
            text: `${groupsUnreadCount.toLocaleString()} unread group message${groupsUnreadCount === 1 ? "" : "s"}`,
            enabled: false,
          }),
        );
      }

      if (lastBackupTime !== undefined) {
        const backupText =
          lastBackupTime === null
            ? "Last backup: never"
            : `Last backup: ${lastBackupTime.toLocaleDateString()} ${lastBackupTime.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`;
        items.push(await MenuItem.new({ text: backupText, enabled: false }));
      }

      items.push(await PredefinedMenuItem.new({ item: "Separator" }));

      // --- Actions section ---
      items.push(
        await MenuItem.new({
          text: isWindowVisible ? "Hide window" : "Show window",
          action: async () => {
            if (await window.isVisible()) {
              await window.hide();
              setIsWindowVisible(false);
            } else {
              await window.unminimize();
              await window.show();
              await window.setFocus();
              setIsWindowVisible(true);
            }
          },
        }),
      );

      if (maybeOpenVaultId && !isSoftLocked) {
        items.push(
          await MenuItem.new({
            text: "Lock now",
            action: () => {
              onLockNow();
            },
          }),
        );

        items.push(
          await MenuItem.new({
            text: "Show logs",
            action: async () => {
              const label = "logs";
              const maybeExistingWindow = await WebviewWindow.getByLabel(label);
              if (maybeExistingWindow) {
                await maybeExistingWindow.show();
              } else {
                const webview = new WebviewWindow(label, {
                  url: "logs.html",
                  title: "Logs (Sentinel)",
                });
                await webview.once("tauri://error", (e) => {
                  console.error("Error creating webview window:", e);
                });
              }
            },
          }),
        );
      }

      items.push(await PredefinedMenuItem.new({ item: "Separator" }));

      items.push(
        await MenuItem.new({
          text: "Exit Sentinel and Background Helper",
          action: () => {
            fullyExitApp();
          },
        }),
      );

      const menu = await Menu.new({ items });
      await tray.setMenu(menu);
    };

    buildMenu();
  }, [
    maybeTrayPromise,
    maybeOpenVaultId,
    isSoftLocked,
    sourceMessageUnreadCount,
    groupsUnreadCount,
    lastBackupTime,
    isWindowVisible,
    JSON.stringify(activeBackgroundTaskNames),
  ]);

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

      if (isHung) {
        ctx.font = `${width}px serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText("⚠️", width / 2, height / 2);
      }

      const afterImageData = ctx.getImageData(0, 0, width, height);

      if (maybeOpenVaultId) {
        await tray.setTooltip(`${await getName()} (${maybeOpenVaultId})`);
      } else {
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
    maybeOpenVaultId,
    hasUnreadMessages,
    isImportantStuffInProgress,
    isHung,
  ]);
};
