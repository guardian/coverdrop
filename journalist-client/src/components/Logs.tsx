import {
  EuiBasicTable,
  EuiButton,
  EuiButtonIcon,
  EuiFieldSearch,
  EuiFlexGroup,
  EuiFlyoutBody,
  EuiFlyoutHeader,
  EuiHealth,
  EuiProgress,
  EuiSelect,
  EuiLoadingSpinner,
  EuiHighlight,
  useEuiTheme,
  EuiDatePicker,
  EuiFlyout,
  EuiTimeline,
  EuiText,
  EuiLink,
  EuiIcon,
} from "@elastic/eui";
import { UIEventHandler, useEffect, useMemo, useRef, useState } from "react";
import { getLoggingSessionsTimeline, getLogs } from "../commands/admin.ts";
import { useStateDebounced } from "../hooks/useStateDebounced.ts";
import moment from "moment";
import { LogEntry } from "../model/bindings/LogEntry.ts";
import { LoggingSession } from "../model/bindings/LoggingSession.ts";

const DATETIME_FORMAT = "YYYY-MM-DD h:mmA";

const LEVELS_AND_COLOURS: Record<string, string> = {
  TRACE: "subdued",
  DEBUG: "accent",
  INFO: "primary",
  WARN: "warning",
  ERROR: "danger",
} as const;

const LEVELS = Object.keys(LEVELS_AND_COLOURS);

const DEFAULT_LOGS_PAGE_SIZE = 33;

interface SessionDivider {
  nextSessionId: bigint | null;
}
function isSessionDivider(
  item: LogEntry | SessionDivider,
): item is SessionDivider {
  return "nextSessionId" in item;
}

export const Logs = () => {
  const [before, setBefore] = useState<Date>();

  const [sessionsTimeline, setSessionsTimeline] = useState<LoggingSession[]>(
    [],
  );
  useEffect(() => {
    getLoggingSessionsTimeline().then(setSessionsTimeline);
  }, []);
  const maybeCurrentSessionId = useMemo(
    () => sessionsTimeline[0]?.sessionId,
    [sessionsTimeline],
  );
  const [shouldShowSessionsTimeline, setShouldShowSessionsTimeline] =
    useState(false);

  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const [logLevel, setLogLevel] = useState<string>("INFO");
  const [immediateSearchTerm, debouncedSearchTerm, setSearchTerm] =
    useStateDebounced("", 250);

  const [isTailing, setIsTailing] = useState(false);
  useEffect(() => {
    const tailingInterval = setInterval(() => {
      if (isTailing) {
        setBefore(new Date());
      }
    }, 500);
    return () => clearInterval(tailingInterval);
  }, [isTailing]);

  const [currentPage, setCurrentPage] = useState(0);

  const scrollableRef = useRef<HTMLDivElement>(null);

  const loadLogs = (shouldAppend: boolean) => {
    setIsLoading(true);
    getLogs({
      minLevel: logLevel,
      searchTerm: debouncedSearchTerm,
      before: before ?? new Date(),
      limit: DEFAULT_LOGS_PAGE_SIZE,
      offset: shouldAppend ? currentPage * DEFAULT_LOGS_PAGE_SIZE : 0,
    })
      .then((newLogs) => {
        if (!shouldAppend && !isTailing && scrollableRef.current) {
          scrollableRef.current.scrollTop = 0;
        }
        setCurrentPage((prev) => (shouldAppend ? prev + 1 : 1));
        setLogs((prevLogs) =>
          shouldAppend ? [...prevLogs, ...newLogs] : newLogs,
        );
      })
      .catch((e) => {
        // TODO consider error toast
        console.error(e);
      })
      .finally(() => setIsLoading(false));
  };

  const refresh = () => setBefore(new Date());
  useEffect(refresh, [logLevel, debouncedSearchTerm]);
  useEffect(() => {
    loadLogs(false);
  }, [before]);

  useEffect(() => {
    if (new Set(logs.map((_) => _.id)).size < logs.length) {
      // TODO should show toast
      // TODO use the ROWIDs to actually identify the dupes
      console.error("dupe logs detected");
    }
  }, [logs]);

  const logsWithSessionDividers = useMemo(
    () =>
      logs.reduce(
        (acc, log, index) => {
          const maybeNextEntry = logs[index + 1];
          return [
            ...(maybeCurrentSessionId &&
            index === 0 &&
            log.sessionId !== maybeCurrentSessionId
              ? [
                  {
                    nextSessionId: log.sessionId,
                  },
                ]
              : acc),
            log,
            ...(maybeNextEntry && log.sessionId !== maybeNextEntry.sessionId
              ? [{ nextSessionId: maybeNextEntry.sessionId }]
              : []),
          ];
        },
        [] as Array<LogEntry | SessionDivider>,
      ),
    [logs],
  );

  const onScroll: UIEventHandler<HTMLDivElement> = ({ currentTarget }) => {
    if (
      !isLoading &&
      currentTarget.scrollHeight - currentTarget.scrollTop <
        currentTarget.clientHeight + 50 /* grace pixels */
    ) {
      loadLogs(true);
    }
  };

  const { euiTheme } = useEuiTheme();

  return (
    <>
      <EuiFlyoutHeader>
        <EuiFlexGroup dir="row" alignItems="center">
          <EuiButton onClick={() => setIsTailing((prev) => !prev)}>
            {isTailing ? (
              <>
                <EuiLoadingSpinner /> Stop Tailing
              </>
            ) : (
              "Start Tailing"
            )}
          </EuiButton>
          {!isTailing && (
            <>
              <EuiButtonIcon
                iconType="refresh"
                aria-label="Refresh Logs"
                onClick={refresh}
              ></EuiButtonIcon>
              <EuiFlexGroup dir="row" alignItems="center" gutterSize="xs">
                Before:
                <EuiDatePicker
                  css={{ width: "max-content" }}
                  dateFormat={DATETIME_FORMAT}
                  minDate={
                    sessionsTimeline.length === 0
                      ? undefined
                      : moment(
                          sessionsTimeline.map((_) => _.minTimestamp).sort()[0],
                        )
                  }
                  maxDate={moment()}
                  selected={moment(before)}
                  onChange={(timestamp) => {
                    if (timestamp) {
                      setIsTailing(false);
                      setBefore(timestamp.toDate());
                    }
                  }}
                  showTimeSelect
                />
              </EuiFlexGroup>
            </>
          )}
          <EuiSelect
            fullWidth={false}
            options={LEVELS.map((level) => ({ value: level, text: level }))}
            value={logLevel}
            onChange={({ target }) => {
              setLogLevel(target.value);
            }}
          />
          <EuiFieldSearch
            placeholder="Search log messages..."
            value={immediateSearchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            fullWidth
            aria-label="Search logs"
            autoCapitalize="off"
            autoCorrect="off"
          />
          {sessionsTimeline.length > 0 && (
            <EuiButton
              minWidth="max-content"
              iconType="timeline"
              onClick={() => setShouldShowSessionsTimeline(true)}
            >
              View Sessions
            </EuiButton>
          )}
        </EuiFlexGroup>
      </EuiFlyoutHeader>

      <EuiFlyoutBody>
        <div
          ref={scrollableRef}
          style={{
            margin: "24px",
            overflowY: "scroll",
            position: "absolute",
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
          }}
          onScroll={onScroll}
        >
          <EuiBasicTable<LogEntry | SessionDivider>
            tableLayout="auto"
            items={logsWithSessionDividers}
            columns={[
              {
                field: "sessionId",
                name: "Session",
                width: "min-content",
              },
              {
                field: "timestamp",
                name: "Timestamp",
                width: "max-content",
                render: (timestamp: string | undefined) =>
                  timestamp && new Date(timestamp).toLocaleString(),
              },
              {
                field: "level",
                name: "Level",
                width: "min-content",
                render: (level: string, item) =>
                  "level" in item && (
                    <EuiHealth
                      style={{ whiteSpace: "nowrap" }}
                      color={LEVELS_AND_COLOURS[level] ?? "hotpink"}
                    >
                      {level}
                    </EuiHealth>
                  ),
              },
              {
                field: "target",
                name: "Target",
                width: "max-content",
                render: (target: string | undefined) =>
                  target && (
                    <EuiHighlight search={immediateSearchTerm}>
                      {target}
                    </EuiHighlight>
                  ),
              },
              {
                field: "message",
                name: "Message",
                render: (message: string | undefined, item) =>
                  isSessionDivider(item) ? (
                    <div
                      style={{
                        width: "100%",
                        textAlign: "right",
                        fontStyle: "italic",
                        color: "black",
                      }}
                    >
                      <EuiIcon type="menuDown" size="l" /> PREVIOUS SESSION (
                      {item.nextSessionId?.toLocaleString()})
                    </div>
                  ) : (
                    message && (
                      <EuiHighlight search={immediateSearchTerm}>
                        {message}
                      </EuiHighlight>
                    )
                  ),
              },
            ]}
            loading={isTailing}
            noItemsMessage={isLoading ? "Loading logs..." : "No logs found"}
            css={{
              th: {
                position: "sticky",
                top: 0,
                background: euiTheme.colors.body,
                zIndex: 98,
              },
            }}
            rowProps={(item) =>
              isSessionDivider(item)
                ? {
                    css: {
                      td: {
                        position: "sticky",
                        top: "30px",
                        zIndex: 99,
                        background: euiTheme.colors.warning,
                      },
                    },
                  }
                : {}
            }
          />
          <div style={{ position: "relative" }}>
            {isLoading && (
              <EuiProgress size="m" color="primary" position="absolute" />
            )}
          </div>
        </div>
      </EuiFlyoutBody>
      {shouldShowSessionsTimeline && (
        <EuiFlyout onClose={() => setShouldShowSessionsTimeline(false)}>
          <EuiFlyoutBody>
            <EuiTimeline
              items={sessionsTimeline.map((session) => ({
                icon: "dot",
                children: (
                  <EuiText>
                    Session{" "}
                    <strong>{session.sessionId.toLocaleString()}</strong>
                    {" - "}
                    {moment(session.minTimestamp).format(
                      DATETIME_FORMAT,
                    )} to{" "}
                    <EuiLink
                      onClick={() => {
                        setShouldShowSessionsTimeline(false);
                        setBefore(new Date(session.maxTimestamp));
                      }}
                    >
                      {moment(session.maxTimestamp).format(DATETIME_FORMAT)}
                    </EuiLink>
                  </EuiText>
                ),
              }))}
            />
          </EuiFlyoutBody>
        </EuiFlyout>
      )}
    </>
  );
};
