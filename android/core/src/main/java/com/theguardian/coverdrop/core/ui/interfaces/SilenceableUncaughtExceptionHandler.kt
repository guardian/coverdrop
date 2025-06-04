package com.theguardian.coverdrop.core.ui.interfaces

/**
 * An exception handler hat can be silenced by acquiring a
 * [UncaughtExceptionHandlerProxySilenceTicket]. While the ticket lock is held, any crash will
 * simply exit the current process which kills the top-most activity (i.e. the [CoverDropActivity]).
 */
interface SilenceableUncaughtExceptionHandler {

    /**
     * Acquires a [UncaughtExceptionHandlerProxySilenceTicket]. White it is being held, all
     * crashes lead to a clean exit of the current process without further error reporting.
     *
     * This should be called in the [Activity.onCreate] method as the very first thing.
     */
    fun acquireSilenceTicket(): UncaughtExceptionHandlerProxySilenceTicket
}

/**
 * A lock that while held causes crashes to close the app silently
 */
interface UncaughtExceptionHandlerProxySilenceTicket {

    /**
     * Releases this [UncaughtExceptionHandlerProxySilenceTicket].
     *
     * This should be called in the [Activity.onDestroy] method as the very last thing.
     */
    fun release()
}
