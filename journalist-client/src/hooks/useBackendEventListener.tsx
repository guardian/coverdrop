import { EventType } from "../model/bindings/EventType.ts";
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

export const useBackendEventListener = (eventName: EventType) => {
  const [data, setData] = useState<{
    remainingCount: number | null;
    startingCount: number;
    lastReceivedAt?: Date;
  }>({ remainingCount: 0, startingCount: 0 });

  useEffect(() => {
    const unlistenFnPromise = listen<number | null>(eventName, (event) => {
      console.log("Event received from backend", event);
      setData((prev) => {
        return {
          remainingCount: event.payload,
          startingCount:
            event.payload === 0
              ? 0
              : Math.max(event.payload ?? 0, prev.startingCount),
          lastReceivedAt: new Date(),
        };
      });
    });
    return () => {
      unlistenFnPromise.then((unlisten) => unlisten());
    };
  }, [eventName]);
  return data;
};
