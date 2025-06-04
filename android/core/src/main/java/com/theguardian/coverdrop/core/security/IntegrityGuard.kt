package com.theguardian.coverdrop.core.security

import android.app.KeyguardManager
import android.content.Context
import android.content.pm.ApplicationInfo
import android.os.Debug
import android.view.MotionEvent
import com.scottyab.rootbeer.RootBeer
import java.util.EnumSet


/**
 * This class implements a few methods suggested by the MASTG handbook. As mentioned there: "The
 * lack of any of these measures does not cause a vulnerability - instead, they are meant to
 * increase the app's resilience against reverse engineering and specific client-side attacks."
 *
 * See: https://mas.owasp.org/MASTG/Android/0x05j-Testing-Resiliency-Against-Reverse-Engineering/
 */
class IntegrityGuard private constructor() {

    /**
     * Set of all integrity violations that have been observed so far. Note that we only add to
     * this set and never remove.
     */
    private val violations: EnumSet<IntegrityViolation> =
        EnumSet.noneOf(IntegrityViolation::class.java)

    /**
     * When set to a non-empty set, the user has snoozed the included integrity violations.
     */
    private val snoozedViolations: EnumSet<IntegrityViolation> =
        EnumSet.noneOf(IntegrityViolation::class.java)

    private val callbacks = ArrayList<IntegrityViolationCallback>()

    /**
     * Adds the given [IntegrityViolation] to a set of snoozed violations that are effectively
     * subtracted from the set of observed integrity violations while this object exists.
     */
    fun snooze(ignoreViolations: EnumSet<IntegrityViolation>) {
        snoozedViolations.addAll(ignoreViolations)
        onViolationsChanged()
    }

    /**
     * Checks whether a touch event might be affected by any overlays that could be indicative of
     * tricking the user.
     *
     * See: https://mas.owasp.org/MASTG/tests/android/MASVS-PLATFORM/MASTG-TEST-0035/
     */
    fun checkMotionEvent(event: MotionEvent) {
        if (event.flags and MotionEvent.FLAG_WINDOW_IS_OBSCURED != 0) {
            addViolation(IntegrityViolation.OVERLAPPED_WINDOW)
        } else if (event.flags and MotionEvent.FLAG_WINDOW_IS_PARTIALLY_OBSCURED != 0) {
            addViolation(IntegrityViolation.OVERLAPPED_WINDOW)
        }
    }

    /**
     * Checks that the device has a secure screen lock (e.g. using PIN, pattern, or password).
     *
     * See: https://mas.owasp.org/MASTG/tests/android/MASVS-STORAGE/MASTG-TEST-0012/
     * Also see:
     */
    fun checkForScreenLock(context: Context) {
        val keyguardManager = context.getSystemService(Context.KEYGUARD_SERVICE) as KeyguardManager
        if (!keyguardManager.isDeviceSecure) {
            addViolation(IntegrityViolation.NO_SCREEN_LOCK)
        }
    }

    /**
     * Checks whether the device might be rooted. A rooted device comes with less guarantees about
     * the overall device software state. Uses the rootbeer library.
     *
     * See: https://mas.owasp.org/MASTG/Android/0x05j-Testing-Resiliency-Against-Reverse-Engineering/#programmatic-detection
     */
    fun checkForRoot(context: Context) {
        val rootBeer = RootBeer(context)
        rootBeer.setLogging(false)
        if (rootBeer.isRooted || rootBeer.detectRootCloakingApps()) {
            addViolation(IntegrityViolation.DEVICE_ROOTED)
        }
    }

    /**
     * Checks whether the app is debuggable or there is an active debugger. This should never be
     * true for the release app and can be indicative of a patched version.
     *
     * See: https://mas.owasp.org/MASTG/Android/0x05j-Testing-Resiliency-Against-Reverse-Engineering/#programmatic-detection
     */
    fun checkForDebuggable(context: Context) {
        val applicationInfo = context.applicationContext.applicationInfo
        if (applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE != 0) {
            addViolation(IntegrityViolation.DEBUGGABLE)
        }

        if (Debug.isDebuggerConnected()) {
            addViolation(IntegrityViolation.DEBUGGER_DETECTED)
        }
    }

    /**
     * Adds a new [IntegrityViolationCallback]. If there are any active [IntegrityViolation] that
     * were found beforehand, the new callback is called immediately.
     */
    fun addIntegrityViolationCallback(callback: IntegrityViolationCallback) {
        synchronized(callbacks) {
            if (!callbacks.contains(callback)) {
                callbacks.add(callback)

                // if there are any active, non-snoozed violations, let the new callback know
                val effectiveViolations = getEffectiveViolationsSet()
                if (effectiveViolations.isNotEmpty()) {
                    callback.onViolationsChanged(violations)
                }
            }
        }
    }

    /**
     * Removes an existing [IntegrityViolationCallback]
     */
    fun removeIntegrityViolationsCallback(callback: IntegrityViolationCallback) {
        synchronized(callbacks) {
            if (callbacks.contains(callback)) {
                callbacks.remove(callback)
            }
        }
    }

    private fun addViolation(violation: IntegrityViolation) {
        val violationSetChanged = violations.add(violation)
        if (violationSetChanged) {
            onViolationsChanged()
        }
    }

    /**
     * Returns the current effective current set
     */
    private fun getEffectiveViolationsSet(): EnumSet<IntegrityViolation> {
        return if (snoozedViolations.isEmpty()) {
            violations
        } else {
            val result = violations.clone()
            result.removeAll(snoozedViolations)
            result
        }
    }

    private fun onViolationsChanged() {
        synchronized(callbacks) {
            val effectiveViolations = getEffectiveViolationsSet()
            callbacks.forEach { it.onViolationsChanged(effectiveViolations) }
        }
    }

    companion object {
        val INSTANCE = IntegrityGuard()
    }
}

interface IntegrityViolationCallback {
    fun onViolationsChanged(violations: EnumSet<IntegrityViolation>)
}

enum class IntegrityViolation(val description: String) {
    DEVICE_ROOTED("This Android device appears might have been rooted."),
    NO_SCREEN_LOCK("This Android device appears to have no secure screen lock."),
    OVERLAPPED_WINDOW("Another window or overlay might obstruct this app."),
    DEBUGGABLE("The app is debuggable."),
    DEBUGGER_DETECTED("An active debugger is attached to this process."),
}
