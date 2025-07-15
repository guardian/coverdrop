## Generating SPKI-SHA256-BASE64 for Certificate Authority pinning

The SPKI-SHA256-BASE64 were generated using :

```
cat <cert_path> | \
openssl x509 -inform pem -noout -outform pem -pubkey | \
openssl pkey -pubin -inform pem -outform der | \
openssl dgst -sha256 -binary | \
openssl enc -base64
```

by using the 2 root certs in the android code base

```
android/app/src/main/res/raw/trusted_root_global_sign.pem
android/app/src/main/res/raw/trusted_root_amazon.pem
```

`android/app/src/main/res/raw/trusted_root_global_sign.pem` is used for `code.dev-guardianapis.com`
`android/app/src/main/res/raw/trusted_root_amazon.pem` is used for `code.dev-gutools.co.uk`

These are added in Info.plist as pinned CA's

See https://developer.apple.com/news/?id=g9ejcf8y for more details

## App lifecycle

The CoverDrop backend service is initialized separately from the UI. 
When the hosting app launches, it should call `coverDropService.didLaunch(config: config)` 
normally this is done from `func application(_: UIApplication, didFinishLaunchingWithOptions: ..)`

As iOS apps do not terminate that often, we also control some lifecycle events from the 
app foreground `applicationWillEnterForeground(_: UIApplication)` and background `applicationDidEnterBackground(_: UIApplication)` events.

An example of this can be found in [AppDelegate](https://github.com/guardian/coverdrop-internal/blob/main/ios/reference/reference/AppDelegate.swift) 
for the reference app. In here we can remotely enabled and disable the coverdrop service, and can recover from some startup errors, like being offline when 
first using coverdrop.

The diagram below shows how the iOS lifecycle events interact with the coverdrop service.

```mermaid
stateDiagram-v2
     direction TB
  state Foregrounded {
    direction TB
    Active --> Inactive:applicationWillResignActive
    Inactive --> Active:applicationDidBecomeActive
    Active
    Inactive
  }
  state Backgrounded {
    direction TB
    Background
  }
  state CoverDrop {
    direction TB
    CoverDropNotInitialised --> CoverDropFailedToInitialise:StartInitialising
    CoverDropFailedToInitialise --> CoverDropInitialising:RetryInitialising
    CoverDropInitialised --> CoverDropNotInitialised:StopService
    CoverDropInitialising --> CoverDropInitialised
    CoverDropNotInitialised
    CoverDropFailedToInitialise
    CoverDropInitialising
    CoverDropInitialised
  }
  ApplicationWillEnterForeground --> CoverDrop
  Suspended --> NotRunning:applicationWillTerminate
  Suspended --> Background
  Background --> Suspended
  Background --> ApplicationWillEnterForeground
  ApplicationWillEnterForeground --> Inactive
  Inactive --> applicationDidEnterBackground
  applicationDidEnterBackground --> Background
    ```
