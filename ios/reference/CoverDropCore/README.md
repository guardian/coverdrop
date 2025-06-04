# CoverDropCore

This package contains the main cryptographic primitives for CoverDrop.

There are a few additional steps required when integrating this library:

1. Integrate the `CoverDropCore` implementation or equivalents of all available `UIApplicationDelegate` system callback methods. These include:

```
    public static func didLaunch() async throws
    
    public static func didEnterForeground()
    
    public static func didEnterBackground()
    
```
