package com.theguardian.coverdrop.core.utils

import com.google.common.truth.Truth.assertThat
import org.junit.Assert.fail

fun expectThrows(exceptionClass: Class<out java.lang.Exception>, block: () -> Unit) {
    try {
        block()
        fail("Expected exception $exceptionClass to be thrown, but no exception was thrown")
    } catch (e: Exception) {
        assertThat(e).isInstanceOf(exceptionClass)
    }
}
