package com.theguardian.coverdrop.core.persistence

/**
 * Separate namespaces for the actual app integration [LIVE] and testing [TEST] allow running the
 * instrumentation tests on real devices without affecting an existing installed app.
 */
internal enum class CoverDropNamespace(val value: String) {
    LIVE("live"),
    TEST("test"),
}
