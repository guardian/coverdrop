import Foundation
import IOSSecuritySuite
import LocalAuthentication

public struct IntegrityViolations: OptionSet {
    public let rawValue: Int

    public init(rawValue: Int) {
        self.rawValue = rawValue
    }

    public static let deviceJailbroken = IntegrityViolations(rawValue: 1 << 0)
    public static let debuggerDetected = IntegrityViolations(rawValue: 1 << 1)
    public static let passcodeNotSet = IntegrityViolations(rawValue: 1 << 2)
    public static let emulatorDetected = IntegrityViolations(rawValue: 1 << 3)
    public static let reverseEngineeringDetected = IntegrityViolations(rawValue: 1 << 4)

    public var message: String {
        switch self {
        case .debuggerDetected: return "An active debugger is attached to this process"
        case .deviceJailbroken: return "This device appears to be Jailbroken"
        case .passcodeNotSet: return "This device has not got a passcode set"
        case .emulatorDetected: return "This is running in an emulator"
        case .reverseEngineeringDetected: return "This device has evidence of reverse engineering"
        default: return "Unknown error"
        }
    }
}

public class SecuritySuite: ObservableObject {
    public static let shared = SecuritySuite()

    private init() {}
    ///
    /// Set of all integrity violations that have been observed so far. Note that we only add to
    /// this set and never remove.
    ///
    @Published public var violations: IntegrityViolations = []
    ///
    /// When set to a non-empty set, the user has snoozed the included integrity violations.
    ///
    @Published public var snoozedViolations: IntegrityViolations = []

    /// Adds the given [IntegrityViolation] to a set of snoozed violations that are effectively
    /// subtracted from the set of observed integrity violations while this object exists.
    ///
    public func snooze(ignoreViolations: IntegrityViolations) {
        snoozedViolations.formUnion(ignoreViolations)
    }

    /// Checks whether the device might be jailbroken. A jailbroken device comes with less guarantees about
    /// the overall device software state.
    /// See: https://mas.owasp.org/MASTG/tests/ios/MASVS-RESILIENCE/MASTG-TEST-0088/

    public func checkForJailbreak() async {
        if IOSSecuritySuite.amIJailbroken() {
            await addViolation(violation: .deviceJailbroken)
        }
    }

    /// Checks whether the app is debuggable or there is an active debugger. This should never be
    /// true for the release app and can be indicative of a patched version.
    /// https://mas.owasp.org/MASTG/tests/ios/MASVS-RESILIENCE/MASTG-TEST-0089/

    public func checkForDebuggable() async {
        if IOSSecuritySuite.amIDebugged() {
            await addViolation(violation: .debuggerDetected)
        }
    }

    /// Checks if the user has set a passphrase. Users device is more secure if they have set a passphrase.
    /// We want to remind them of this before they use CoverDrop

    public func checkForPassphrase() async {
        if !LAContext().canEvaluatePolicy(.deviceOwnerAuthentication, error: nil) {
            await addViolation(violation: .passcodeNotSet)
        }
    }

    /// Checks if the device is running in an emulator. This is to warn people of using CoverDrop in a emulator
    /// See: https://mas.owasp.org/MASTG/tests/ios/MASVS-RESILIENCE/MASTG-TEST-0092/
    ///
    public func checkForEmulator() async {
        if IOSSecuritySuite.amIRunInEmulator() {
            await addViolation(violation: .emulatorDetected)
        }
    }

    /// Checks whether the device might be reverse engineered.`
    /// See: https://mas.owasp.org/MASTG/tests/ios/MASVS-RESILIENCE/MASTG-TEST-0091/
    ///
    public func checkForReverseEngineering() async {
        if IOSSecuritySuite.amIReverseEngineered() {
            await addViolation(violation: .reverseEngineeringDetected)
        }
    }

    @MainActor public func addViolation(violation: IntegrityViolations) {
        violations.insert(violation)
    }

    ///
    /// Returns the current effective current set
    ///
    public func getEffectiveViolationsSet() -> IntegrityViolations {
        if snoozedViolations.isEmpty {
            return violations
        } else {
            return violations.subtracting(snoozedViolations)
        }
    }
}
