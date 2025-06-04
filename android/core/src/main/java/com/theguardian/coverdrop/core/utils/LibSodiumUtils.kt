package com.theguardian.coverdrop.core.utils

/**
 * Check a LibSodium return value for success (0).
 */
internal fun checkLibSodiumSuccess(res: Int) {
    check(res == 0) { "lib-sodium call failed: res=$res" }
}
