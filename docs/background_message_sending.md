# Background message sending

## iOS

Background message sending is only triggered by background scheduled tasks using the `BGAppRefreshTaskRequest` https://developer.apple.com/documentation/backgroundtasks/choosing-background-strategies-for-your-app#Update-Your-Apps-Content

We schedule background-scheduled tasks when the app enters the background and when the app starts.
We do this extra scheduling at the app start with an extra delay just in case the onAppFinish function does not get called.

If the app starts and message sending work is listed as pending, we try and dequeue and send messages immediately.
As the `BackgroundMessageSendService.run` checks for a) recent run, the message sending may be rate limited, and not run this time.
If the app starts, but no work is pending, we will and add another background task to run in 10 min + an exponential delay.

## Diagram

```mermaid
flowchart TD
    subgraph "App Lifecycle"
        AppStart["App Starts"]
        AppExit["App Exits/About to Exit"]
    end

    subgraph "Background Work Manager"
        OnStart["onAppStart()"]
        OnFinish["onAppFinished()"]
        CheckPending{"Is work pending?"}
        ScheduleWork["Schedule work with exponential delay between 5-120 mins)"]
        SetPendingFlag["Set background work pending flag"]
        ScheduleFailsafe["Schedule failsafe work (10 mins delay + exponential delay between 5-120 mins)"]
        Exit["Exit"]
    end

    subgraph "Background Worker"
        Worker["CoverDropBackgroundWorker starts"]
        CheckLastRun{"Should execute based on last run?"}
        ProcessQueue["Process message queue (up to configured limit)"]
        Success["Mark work as complete Cancel pending work"]
        Retry["Schedule retry with exponential backoff"]
        Exit["Exit"]
    end

     subgraph "State"
        BackgroundWorkPending["BackgroundWorkPending: Bool"]
        BackgroundTaskSuccessfulRun["BackgroundTaskSuccessfulRun: Timestamp"]

     end


    %% App Lifecycle Flow
    AppStart --> OnStart
    AppExit --> OnFinish

    %% Background Work Manager Flow
    BackgroundWorkPending --> CheckPending
    OnStart --> CheckPending
    CheckPending -->|"Yes"| Worker
    OnStart --> ScheduleFailsafe
    ScheduleFailsafe --> SetPendingFlag
    SetPendingFlag --> BackgroundWorkPending
    SetPendingFlag --> Exit
    OnFinish --> ScheduleWork
    ScheduleWork --> SetPendingFlag

    %% Background Worker Flow
    Worker --> CheckLastRun
    CheckLastRun -->|"Yes"| ProcessQueue
    CheckLastRun -->|"No"| Exit
    ProcessQueue -->|"Success"| Success
    Success --> Exit
    Success --> BackgroundTaskSuccessfulRun
    Success --> BackgroundWorkPending
    BackgroundTaskSuccessfulRun --> CheckLastRun
    ProcessQueue -->|"Error"| Retry
    Retry --> Exit

    %% Styling
    classDef process fill:#e1f5fe,stroke:#01579b
    classDef decision fill:#fff3e0,stroke:#ff6f00
    classDef event fill:#f3e5f5,stroke:#4a148c
    classDef state fill:#f3e5,stroke:#4a148c

    class AppStart,AppExit event
    class CheckPending,CheckLastRun decision
    class Worker,ProcessQueue,ScheduleWork,Exit process
    class BackgroundWorkPending,BackgroundTaskSuccessfulRun state
```

## Functions

### `BackgroundMessageSendService.run`

This tries to dequeue and send background messages
This function implements rate limiting, so will only run once every `minDurationBetweenBackgroundRunsInSecs` which is set to once per hour in production
It also supports the case where the clock has jumped forwards, so will run in this scenario too.

### ``

### `BackgroundMessageScheduleService.onAppStart`

This will fire every time the app starts from a cold start - ie the app is not currently running in the background.
This is done by putting a call in `didFinishLaunchingWithOptions` app delegate function.

### `BackgroundMessageScheduleService.scheduleBackgroundTask`

This is called when the app enters the background from `applicationDidEnterBackground` in app delegate
This will overwrite any previously scheduled background tasks with this most recent one

## State

We store 2 state values related to background message sending
`BackgroundWorkPending` - This is set in `PublicDataRepository.shared.writeBackgroundWorkPending()` function which is backed by a `UserDefaults` value stored on the device
`BackgroundJobLastSuccessfulRun` - This is set in `PublicDataRepository.shared.writeBackgroundJobLastSuccessfulRun()` function which is backed by a `UserDefaults` value stored on the device

```

```
