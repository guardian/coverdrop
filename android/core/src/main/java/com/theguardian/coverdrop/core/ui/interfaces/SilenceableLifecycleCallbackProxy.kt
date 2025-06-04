package com.theguardian.coverdrop.core.ui.interfaces

/**
 * An activity lifecycle callback that can be silenced by acquiring a
 * [LifecycleCallbackProxySilenceTicket]. While the ticket lock is held, all callback are ignored globally.
 */
interface SilenceableLifecycleCallbackProxy {

    /**
     * Acquires a [LifecycleCallbackProxySilenceTicket]. White it is being held, all
     * [Application.ActivityLifecycleCallbacks] as being ignored.
     *
     * This must be called in the [Activity.onCreate] method as the very first thing.
     */
    fun acquireSilenceTicket(): LifecycleCallbackProxySilenceTicket
}

/**
 * A lock that while held causes all activity callbacks to libraries to be ignored application-wide.
 */
interface LifecycleCallbackProxySilenceTicket {

    /**
     * Releases this [LifecycleCallbackProxySilenceTicket].
     *
     * This must be called in the [Activity.onCreate] method as the very last thing.
     */
    fun release()
}
