package com.theguardian.coverdrop.testutils

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class TestVectorsCryptoTest {

    private val context = InstrumentationRegistry.getInstrumentation().targetContext

    @Test
    fun readJson_whenGivenValidFileName_thenReturnsNonEmptyResult() {
        val testVectors = TestVectors(context, "certificate_data")
        assertTrue(testVectors.readFile("04_certificate_data").isNotEmpty())
    }
}
