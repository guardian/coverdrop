package com.theguardian.coverdrop.core.persistence

import com.theguardian.coverdrop.core.persistence.CoverDropNamespace.LIVE
import com.theguardian.coverdrop.core.persistence.CoverDropNamespace.TEST


/**
 * Separate namespaces for the actual app integration [LIVE] and testing [TEST] allow running the
 * instrumentation tests on real devices without affecting an existing installed app.
 */
internal enum class CoverDropNamespace(val value: String) {
    LIVE("live"),
    TEST("test"),
}
