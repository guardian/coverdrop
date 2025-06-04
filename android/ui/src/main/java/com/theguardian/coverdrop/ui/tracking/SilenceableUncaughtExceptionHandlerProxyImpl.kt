package com.theguardian.coverdrop.ui.tracking

import android.os.Process
import com.theguardian.coverdrop.core.ui.interfaces.SilenceableUncaughtExceptionHandler
import com.theguardian.coverdrop.core.ui.interfaces.UncaughtExceptionHandlerProxySilenceTicket
import java.lang.Thread.UncaughtExceptionHandler
import java.util.concurrent.atomic.AtomicInteger
import kotlin.system.exitProcess

interface UncaughtExceptionHandlerProxy : UncaughtExceptionHandler {
    fun init(originalExceptionHandler: UncaughtExceptionHandler)
}

class SilenceableUncaughtExceptionHandlerProxyImpl private constructor() :
    UncaughtExceptionHandlerProxy, SilenceableUncaughtExceptionHandler {

    companion object {
        val INSTANCE = SilenceableUncaughtExceptionHandlerProxyImpl()
    }

    private var originalUncaughtExceptionHandler: UncaughtExceptionHandler? = null

    /**
     * The original exception handler is silenced if there is at least one active request to do so.
     */
    private val numSilenceTicketsActive = AtomicInteger(0)

    //
    // SilenceableUncaughtExceptionHandlerProxy
    //

    override fun acquireSilenceTicket(): UncaughtExceptionHandlerProxySilenceTicket {
        checkNotNull(originalUncaughtExceptionHandler) { "Must call `init` first" }

        numSilenceTicketsActive.incrementAndGet()
        return UncaughtExceptionHandlerProxySilenceTicketImpl(numSilenceTicketsActive)
    }

    //
    // SilenceableUncaughtExceptionHandlerProxy
    //

    override fun init(originalExceptionHandler: UncaughtExceptionHandler) {
        originalUncaughtExceptionHandler = originalExceptionHandler
        Thread.setDefaultUncaughtExceptionHandler(this)
    }

    override fun uncaughtException(t: Thread, e: Throwable) {
        if (numSilenceTicketsActive.get() > 0) {
            // If we are silenced, we close the app silently by  asking the system to kill our
            // process and then exiting from this process. This is similar to the behaviour
            // of the default exception handler `KillApplicationHandler`:
            // See https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/com/android/internal/os/RuntimeInit.java;l=174-175
            Process.killProcess(Process.myPid())
            exitProcess(0)
        } else {
            // Otherwise we pass everything to the original (most likely default) exception handler
            originalUncaughtExceptionHandler!!.uncaughtException(t, e)
        }
    }
}

/**
 * Implementation of the [UncaughtExceptionHandlerProxySilenceTicket] that references the [AtomicInteger]
 * used for counting the number of silencing requests.
 *
 * It overrides [finalize] as a safe-guard that is triggered if the app code forgot to release the
 * lock and this object is being garbage collected.
 */
internal class UncaughtExceptionHandlerProxySilenceTicketImpl(private val globalActiveTickets: AtomicInteger) :
    UncaughtExceptionHandlerProxySilenceTicket {

    private var hasBeenReleased = false

    init {
        require(globalActiveTickets.get() >= 1)  // since this is a new active request, this should be at least 1
    }

    override fun release() {
        synchronized(this) {
            // we should not have been released before
            require(!hasBeenReleased)

            // since this is a new active request, this should be at least 1 before we decrement
            require(globalActiveTickets.getAndDecrement() >= 1)

            hasBeenReleased = true
        }
    }

    /**
     * Safe-guard in case [release] is not being called. See the Kotlin documentation that no
     * `override` keyword is needed: https://kotlinlang.org/docs/java-interop.html#finalize
     */
    protected fun finalize() {
        if (hasBeenReleased) return // quick return without lock
        synchronized(this) {
            if (!hasBeenReleased) {
                release()
            }
        }
    }
}
