package com.theguardian.coverdrop.audit

import android.content.ComponentName
import android.content.pm.ActivityInfo
import android.content.pm.PackageManager.GET_META_DATA
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import com.google.common.truth.Truth.assertThat
import dagger.hilt.android.testing.HiltAndroidTest
import org.junit.Test
import org.junit.runner.RunWith


/**
 * Self-audit that checks whether the included activities are vulnerable to the
 * " StrandHogg Attack / Task Affinity Vulnerability" as per
 * https://developer.android.com/privacy-and-security/risks/strandhogg.
 */
@HiltAndroidTest
@RunWith(AndroidJUnit4::class)
class TaskHijackingAuditTest {

    private val targetContext = getInstrumentation().targetContext

    @Test
    fun testMainActivity_notVulnerableToTaskHijacking() {
        assertComponentNotVulnerableToTaskHijacking("com.theguardian.coverdrop.MainActivity")
    }

    @Test
    fun testCoverDropActivity_notVulnerableToTaskHijacking() {
        assertComponentNotVulnerableToTaskHijacking("com.theguardian.coverdrop.ui.activities.CoverDropActivity")
    }

    private fun assertComponentNotVulnerableToTaskHijacking(activityClassName: String) {
        val componentName = ComponentName.createRelative(
            "com.theguardian.coverdrop",
            activityClassName
        )

        val pm = targetContext.packageManager
        val activityInfo = pm.getActivityInfo(componentName, GET_META_DATA)

        assertThat(activityInfo.taskAffinity).isNull()
        assertThat(activityInfo.launchMode).isEqualTo(ActivityInfo.LAUNCH_SINGLE_INSTANCE)
    }
}
