import Foundation

public protocol Role {}

public struct TrustedOrganization: Role {}
public struct Organization: Role {}
public struct CoverNodeProvisioning: Role {}
public struct CoverNodeId: Role {}
public struct CoverNodeMessaging: Role {}
public struct JournalistProvisioning: Role {}
public struct JournalistId: Role {}
public struct JournalistMessaging: Role {}
public struct User: Role {}
