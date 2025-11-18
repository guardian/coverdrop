import { EuiProgress, EuiSpacer } from "@elastic/eui";
import { useEffect, useState } from "react";
import { useBackendEventListener } from "../hooks/useBackendEventListener.tsx";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface BackgroundTaskTrackerWithLoadingBarIfApplicableProps {
  isImportantStuffInProgress: boolean;
  setIsImportantStuffInProgress: (value: boolean) => void;
  maybeHungAt: Date | null;
  setMaybeHungAt: (value: Date | null) => void;
}

export const BackgroundTaskTrackerWithLoadingBarIfApplicable = ({
  isImportantStuffInProgress,
  setIsImportantStuffInProgress,
  maybeHungAt,
  setMaybeHungAt,
}: BackgroundTaskTrackerWithLoadingBarIfApplicableProps) => {
  const [isDetailOpen, setIsDetailOpen] = useState(false);
  const expandDetail = () => setIsDetailOpen(true);
  const collapseDetail = () => setIsDetailOpen(false);

  const backgroundTasks = [
    {
      name: "Sending messages",
      data: useBackendEventListener("outbound_queue_length"),
    },
    {
      name: "Processing dead drops",
      data: useBackendEventListener("dead_drops_remaining"),
    },
  ];

  useEffect(() => {
    console.log("background task updated", backgroundTasks);
    if (maybeHungAt) {
      // recover from hung state when background tasks change
      setMaybeHungAt(null);
    }
    const timeout = setTimeout(async () => {
      setMaybeHungAt(new Date());
      await getCurrentWindow().unminimize();
      await getCurrentWindow().show();
    }, 60_000); // 1 minute timeout to consider the app "hung" (e.g. dead drops should run every 15s)

    // clears the timeout if backgroundTasks change before the timeout completes
    // background tasks can fail (e.g. network), but should still update the frontend,
    // we're looking to catch the actual background tasks no longer running
    return () => clearTimeout(timeout);
  }, [JSON.stringify(backgroundTasks)]); // stringify to check for actual changes, rather than reference changes from re-renders

  const hasActiveBackgroundTasks = backgroundTasks.some(
    ({ data }) => data.remainingCount !== 0,
  );

  useEffect(() => {
    setIsImportantStuffInProgress(hasActiveBackgroundTasks);
    if (!hasActiveBackgroundTasks) {
      collapseDetail(); // this ensures, that if you leave your mouse while tasks are done, the detail remains closed
    }
  }, [hasActiveBackgroundTasks]);

  return (
    isImportantStuffInProgress && (
      <div
        onMouseOver={expandDetail}
        onMouseEnter={expandDetail}
        onMouseLeave={collapseDetail}
      >
        <EuiProgress
          size="l"
          color="primary"
          position="absolute"
          style={{
            zIndex: 99,
            cursor: "help",
          }}
        />
        {isDetailOpen && (
          <div
            style={{
              width: "100%",
              top: "-5px",
              left: "0",
              position: "absolute",
              background: "white",
              zIndex: 98,
              padding: "10px",
              paddingTop: "22px",
              borderRadius: "10px",
              boxShadow: "0 4px 8px rgba(0, 0, 0, 0.1)",
              cursor: "default",
            }}
          >
            {backgroundTasks.map(
              ({ name, data }) =>
                data.remainingCount !== 0 && (
                  <div key={name}>
                    <EuiSpacer size="s" />
                    <strong>{name}</strong>:{" "}
                    {data.remainingCount
                      ? `${data.remainingCount} remaining`
                      : ""}
                    <EuiSpacer size="xs" />
                    <EuiProgress
                      size="s"
                      color="accent"
                      value={
                        data.startingCount && data.remainingCount
                          ? data.startingCount - data.remainingCount
                          : undefined
                      }
                      max={data.startingCount ?? undefined}
                    ></EuiProgress>
                    <EuiSpacer size="s" />
                  </div>
                ),
            )}
          </div>
        )}
      </div>
    )
  );
};
