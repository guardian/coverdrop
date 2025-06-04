import {
  EuiBasicTable,
  EuiButtonIcon,
  EuiFieldSearch,
  EuiFlexGroup,
  EuiFlyout,
  EuiFlyoutBody,
  EuiFlyoutHeader,
  EuiHealth,
  EuiSelect,
  IconColor,
} from "@elastic/eui";
import { useState } from "react";
import { SentinelLogEntry } from "../model/bindings/SentinelLogEntry";

const logLevelIdToNumericLevel = (level: string): number => {
  switch (level) {
    case "trace":
      return 0;
    case "debug":
      return 1;
    case "info":
      return 2;
    case "warn":
      return 3;
    case "error":
      return 4;
    default:
      return 2; // Default to info
  }
};

const logLevelToString = (level: number): string => {
  switch (level) {
    case 0:
      return "TRACE";
    case 1:
      return "DEBUG";
    case 2:
      return "INFO";
    case 3:
      return "WARN";
    case 4:
      return "ERROR";
    default:
      return "UNKNOWN";
  }
};

const logLevelToColor = (level: number): IconColor => {
  switch (level) {
    case 0:
      return "subdued";
    case 1:
      return "accent";
    case 2:
      return "primary";
    case 3:
      return "warning";
    case 4:
      return "danger";
    default:
      return "hotpink";
  }
};

export const LogsPanel = ({
  logs,
  setFlyoutVisible,
  refreshClicked,
}: {
  logs: SentinelLogEntry[];
  setFlyoutVisible: (visible: boolean) => void;
  refreshClicked: () => void;
}) => {
  const [logLevel, setLogLevel] = useState<string>("info");
  const [logLevelFilter, setLogLevelFilter] = useState<number>(2);
  const [searchTerm, setSearchTerm] = useState<string>("");

  const columns = [
    {
      field: "timestamp",
      name: "Timestamp",
      width: "min-content",
      render: (timestamp: string) => {
        const date = new Date(timestamp);

        return date.toLocaleString();
      },
    },
    {
      field: "level",
      name: "Level",
      width: "min-content",
      render: (level: number) => (
        <EuiHealth
          style={{ whiteSpace: "nowrap" }}
          color={logLevelToColor(level)}
        >
          {logLevelToString(level)}
        </EuiHealth>
      ),
    },
    {
      field: "target",
      name: "Target",
    },
    {
      field: "message",
      name: "Message",
    },
  ];

  const filteredLogs = logs
    .filter((log) => log.level >= logLevelFilter)
    .filter(
      (log) =>
        searchTerm === "" ||
        log.message.toLowerCase().includes(searchTerm.toLowerCase()),
    );

  return (
    <EuiFlyout size="l" onClose={() => setFlyoutVisible(false)}>
      <EuiFlyoutHeader>
        <EuiFlexGroup dir="row" alignItems="center">
          <EuiButtonIcon
            iconType="refresh"
            onClick={refreshClicked}
          ></EuiButtonIcon>
          <EuiSelect
            fullWidth={false}
            options={[
              {
                value: "trace",
                text: "TRACE",
              },
              {
                value: "debug",
                text: "DEBUG",
              },
              {
                value: "info",
                text: "INFO",
              },
              {
                value: "warn",
                text: "WARN",
              },
              {
                value: "error",
                text: "ERROR",
              },
            ]}
            value={logLevel}
            onChange={(e) => {
              const level = e.target.value;

              const numericLevel = logLevelIdToNumericLevel(level);

              setLogLevelFilter(numericLevel);
              setLogLevel(level);
            }}
          />
          <EuiFieldSearch
            placeholder="Search log messages..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            fullWidth
            aria-label="Search logs"
          />
        </EuiFlexGroup>
      </EuiFlyoutHeader>

      <EuiFlyoutBody>
        <EuiBasicTable
          tableLayout="auto"
          tableCaption="Demo of EuiBasicTable"
          items={filteredLogs}
          columns={columns}
        />
      </EuiFlyoutBody>
    </EuiFlyout>
  );
};
