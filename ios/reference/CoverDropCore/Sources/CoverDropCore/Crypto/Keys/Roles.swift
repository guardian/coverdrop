import Foundation

public protocol Role {}

public protocol IdentityRole: Role {
    static var idKeyRole: IdKeyRole { get }
}

public struct TrustedOrganization: Role {}
public struct Organization: Role {}

public struct CoverNodeProvisioning: Role {}
public struct CoverNodeId: IdentityRole {
    public static let idKeyRole = IdKeyRole.coverNodeId
}

public struct CoverNodeMessaging: Role {}

public struct JournalistProvisioning: Role {}
public struct JournalistId: IdentityRole {
    public static let idKeyRole = IdKeyRole.journalistId
}

public struct JournalistMessaging: Role {}

public struct User: Role {}
