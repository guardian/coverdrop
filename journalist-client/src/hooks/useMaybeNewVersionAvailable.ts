import { useEffect, useState } from "react";
import { getCreatedTimeOfExecutable, softExitApp } from "../commands/admin.ts";
import moment from "moment";

export const useMaybeNewVersionAvailable = () => {
  const [exeCreatedTimestampAtLaunch, setExeCreatedTimestampAtLaunch] =
    useState<Date>();

  const [maybeNewVersionAvailable, setMaybeNewVersionAvailable] = useState<{
    shouldLockUi: boolean;
  }>();

  useEffect(() => {
    getCreatedTimeOfExecutable().then(setExeCreatedTimestampAtLaunch);
  }, []);

  useEffect(() => {
    if (!exeCreatedTimestampAtLaunch) {
      // need to wait for exeCreatedTimestampAtLaunch to be populated, see other useEffect
      return;
    }

    const process = async () => {
      const exeCreatedTimestampAsOfThisMoment =
        await getCreatedTimeOfExecutable();

      // If the created time has not changed there has been no version update
      if (exeCreatedTimestampAtLaunch === exeCreatedTimestampAsOfThisMoment) {
        return;
      }

      const hours_since_update_available = moment().diff(
        exeCreatedTimestampAsOfThisMoment,
        "hours",
      );

      if (hours_since_update_available > 48) {
        // An update has been available for more than 48 hours. Forcing a restart.
        softExitApp(); // relies on the auto-relaunch feature of the app to restart after exit
      } else if (hours_since_update_available > 24) {
        // An update has been available for more than 24 hours. Locking the UI with a button to restart.
        setMaybeNewVersionAvailable({ shouldLockUi: true });
      } else {
        // An update is available. Showing a banner with a button to restart.
        setMaybeNewVersionAvailable({ shouldLockUi: false });
      }
    };

    process(); // run immediately on mount...
    const interval = setInterval(process, 60_000); // ... then every minute

    return () => clearInterval(interval);
  }, [exeCreatedTimestampAtLaunch]);

  return maybeNewVersionAvailable;
};
