@testable import CoverDropCore
import Sodium
import XCTest

final class SecureRandomTests: XCTestCase {
    func testExponentialDuration_whenGivenExpectedMean_thenObservedMeanAndVarianceMatches() {
        let numberOfItems = 10000
        let expectedMeanDuration = Duration.seconds(30 * 60)

        let samples = Array(repeating: 0, count: numberOfItems).compactMap { _ in
            try? SecureRandomUtils.nextDurationFromExponentialDistribution(
                expectedMeanDuration: expectedMeanDuration,
                atLeastDuration: Duration.seconds(0),
                atMostDuration: Duration.seconds(20000)
            )
        }

        let total = samples.reduce(Duration.zero) { acc, item in
            acc + item
        }

        let mean = Double(total.components.seconds) / Double(numberOfItems)

        let tolerance = Duration.seconds(60)

        XCTAssertGreaterThanOrEqual(
            mean,
            Double(expectedMeanDuration.components.seconds) - Double(tolerance.components.seconds)
        )
        XCTAssertLessThanOrEqual(
            mean,
            Double(expectedMeanDuration.components.seconds) + Double(tolerance.components.seconds)
        )

        let variance = samples.map { (Double($0.components.seconds) - mean) / 60 }
            .map { $0 * $0 }
            .reduce(0, +) / Double(numberOfItems)

        let expectedVarianceMinutes = 1.0 / ((1.0 / 30.0) * (1.0 / 30.0))

        XCTAssertGreaterThanOrEqual(variance, 0.8 * expectedVarianceMinutes)
        XCTAssertLessThanOrEqual(variance, 1.2 * expectedVarianceMinutes)
    }

    func testExponentialDuration_whenGivenBounds_thenResultsAlwaysWithin() {
        let numberOfItems = 100
        let expectedMeanDuration = Duration.seconds(30 * 60)
        let lowerBound = Duration.seconds(40 * 60)
        let upperBound = Duration.seconds(45 * 60)

        let samples = Array(repeating: 0, count: numberOfItems).compactMap { _ in
            try? SecureRandomUtils.nextDurationFromExponentialDistribution(
                expectedMeanDuration: expectedMeanDuration,
                atLeastDuration: lowerBound,
                atMostDuration: upperBound
            )
        }

        for sample in samples {
            XCTAssertGreaterThanOrEqual(sample, lowerBound)
            XCTAssertLessThanOrEqual(sample, upperBound)
        }
    }
}
