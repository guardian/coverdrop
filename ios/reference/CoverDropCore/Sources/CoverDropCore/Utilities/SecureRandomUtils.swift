import Foundation
import Sodium

enum SecureRandomUtilsError: Error {
    case expectedMeanIsZero, cannotCoerceIntoEmptyRange(message: String)
}

enum SecureRandomUtils {
    /**
     * Draws durations from a exponential distribution with the given expected mean (1/lambda).
     */
    static func nextDurationFromExponentialDistribution(
        expectedMeanDuration: Duration,
        atLeastDuration: Duration,
        atMostDuration: Duration
    ) throws -> Duration {
        guard expectedMeanDuration.components.seconds > 0 else {
            throw SecureRandomUtilsError.expectedMeanIsZero
        }
        let lambda = 1.0 / Double(expectedMeanDuration.components.seconds)

        // This is from Sodium, and guaranteed to be cryptographically secure
        // https://github.com/jedisct1/swift-sodium/blob/master/Sodium/RandomBytes.swift
        var rng = RandomBytes.Generator()
        let nextDouble = Double.random(in: 0 ... 1, using: &rng)

        // The subtraction `1.0-...` ensures that we do not call `log` with 0.0
        let randomValue = -1.0 / lambda * log(1.0 - nextDouble)

        let duration = Duration.seconds(randomValue).components.seconds
        return Duration.seconds(coerceIn(
            duration: duration,
            minimumValue: atLeastDuration.components.seconds,
            maximumValue: atMostDuration.components.seconds
        ))
    }

    static func coerceIn(duration: Int64, minimumValue: Int64, maximumValue: Int64) -> Int64 {
        if duration < minimumValue {
            return minimumValue
        } else if duration > maximumValue {
            return maximumValue
        } else {
            return duration
        }
    }
}
