package com.theguardian.coverdrop.core.utils

import java.time.Instant


interface IClock {
    fun now(): Instant
}

/**
 * Default [IClock] implementation that returns the current system time.
 */
class DefaultClock : IClock {
    override fun now(): Instant = Instant.now()
}

private const val NS_IN_MS = 1_000_000

/**
 * [IClock] implementation that is monotonic (i.e. the next value is always greater than the last one).
 */
class MonotonicClock : IClock {
    override fun now(): Instant = Instant.ofEpochMilli(System.nanoTime() / NS_IN_MS)
}
