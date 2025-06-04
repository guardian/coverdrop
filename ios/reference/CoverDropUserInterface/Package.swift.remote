// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "CoverDropUserInterface",
    platforms: [.iOS(.v17)],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "CoverDropUserInterface",
            targets: ["CoverDropUserInterface"]
        )
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        .package(url: "https://github.com/guardian/coverdrop-core-ios", exact: "1.0.0-alpha"),
        .package(url: "https://github.com/exyte/SVGView.git", from: "1.0.4"),
        .package(url: "https://github.com/guardian/fonts.git", branch: "main")
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "CoverDropUserInterface",
            dependencies: [.product(name: "CoverDropCore", package: "coverdrop-core-ios"), "SVGView", .product(name: "GuardianFonts", package: "fonts")],
            resources: [
                .process("Resources/Icons")
            ]
        ),
        .testTarget(
            name: "CoverDropUserInterfaceTests",
            dependencies: ["CoverDropUserInterface", "SVGView", .product(name: "CoverDropCore", package: "coverdrop-core-ios")]
        )
    ]
)
