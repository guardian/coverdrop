import CoverDropCore
import Foundation
import SwiftUI

public struct SecurityAlert: View {
    @ObservedObject var securitySuite = SecuritySuite.shared
    @Environment(\.dismiss) private var dismiss

    public var body: some View {
        HeaderView(type: .about, dismissAction: {
            dismiss()
        }) {
            VStack(alignment: .leading, spacing: Padding.medium) {
                Text("Secure Messaging is not available. We believe that this device might not be secure.")
                    .fontWeight(.semibold)
                getViolationMessages(violations: securitySuite.getEffectiveViolationsSet())
                Text(
                    """
                    If you believe these are false positives,
                    you can ignore this warning for this session using the button below.
                    """
                )
                Spacer()
                Button("Dismiss and ignore warnings") {
                    securitySuite.snooze(ignoreViolations: securitySuite.getEffectiveViolationsSet())
                }.accessibilityIdentifier("Dismiss and ignore warnings")
                    .buttonStyle(SecondaryButtonStyle(isDisabled: false))
            }.padding(Padding.large)
                .foregroundColor(Color.white)
        }
    }

    func getViolationMessages(violations: IntegrityViolations) -> some View {
        return VStack(alignment: .leading, spacing: 10) {
            if violations.contains(.deviceJailbroken) {
                Text("- \(IntegrityViolations.deviceJailbroken.message)")
            }
            if violations.contains(.passcodeNotSet) {
                Text("- \(IntegrityViolations.passcodeNotSet.message)")
            }
            if violations.contains(.reverseEngineeringDetected) {
                Text("- \(IntegrityViolations.reverseEngineeringDetected.message)")
            }
            if violations.contains(.emulatorDetected) {
                Text("- \(IntegrityViolations.emulatorDetected.message)")
            }
            if violations.contains(.debuggerDetected) {
                Text("- \(IntegrityViolations.debuggerDetected.message)")
            }
        }
    }
}
