// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "CoverDropCore",
    platforms: [.iOS(.v17)],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "CoverDropCore",
            targets: ["CoverDropCore"]
        )
    ],

    dependencies: [
        // Dependencies declare other packages that this package depends on.
        .package(
            url: "https://github.com/jedisct1/swift-sodium.git",
            revision: "63240810df971557fe9badc557257bdfbfeb90a3"
        ),
        .package(url: "https://github.com/1024jp/GzipSwift", from: "5.2.0"),
        .package(url: "https://github.com/lambdapioneer/sloth-ios.git", from: "0.3.0"),
        .package(url: "https://github.com/securing/IOSSecuritySuite.git", from: "1.5.0")
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "CoverDropCore",
            dependencies: [
                .product(name: "Sodium", package: "swift-sodium"),
                .product(name: "Gzip", package: "GzipSwift"),
                .product(name: "RainbowSloth", package: "sloth-ios"),
                .product(name: "IOSSecuritySuite", package: "IOSSecuritySuite")
            ],
            resources: [
                .copy("Resources/eff_large_wordlist.txt"),
                .copy("Resources/vectors/"),
                .copy("Resources/keys/"),
                .copy("Resources/organization_keys/")
            ]
        ),
        .testTarget(
            name: "CoverDropCoreTests",
            dependencies: ["CoverDropCore", "IOSSecuritySuite"],
            resources: [
                .copy("Resources/vectors/")
            ]
        )
    ]
)
