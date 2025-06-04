package com.theguardian.coverdrop.testutils

import android.app.Application
import android.content.Context
import androidx.test.runner.AndroidJUnitRunner

/**
 * Custom test runner to inject the mocked application class in place for the actual one.
 */
class CoverDropTestRunner : AndroidJUnitRunner() {

    override fun newApplication(
        cl: ClassLoader?,
        className: String?,
        context: Context?,
    ): Application {
        return super.newApplication(cl, CoverDropTestApplication::class.qualifiedName, context)
    }

}
