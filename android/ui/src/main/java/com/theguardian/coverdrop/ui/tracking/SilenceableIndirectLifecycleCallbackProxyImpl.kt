package com.theguardian.coverdrop.ui.tracking

import android.app.Activity
import android.app.Application.ActivityLifecycleCallbacks
import android.os.Bundle
import com.theguardian.coverdrop.core.ui.interfaces.LifecycleCallbackProxySilenceTicket
import com.theguardian.coverdrop.core.ui.interfaces.SilenceableLifecycleCallbackProxy
import com.theguardian.coverdrop.ui.tracking.SilenceableIndirectLifecycleCallbackProxyImpl.Companion.INSTANCE
import java.util.concurrent.atomic.AtomicInteger

/**
 * An indirect [ActivityLifecycleCallbacks] implementation that forwards the callbacks to the
 * registered children (similar to the implementation in [android.app.Application].
 */
interface IndirectLifecycleCallbackProxy : ActivityLifecycleCallbacks {
    fun registerActivityLifecycleCallbacks(callback: ActivityLifecycleCallbacks?)
    fun unregisterActivityLifecycleCallbacks(callback: ActivityLifecycleCallbacks?)
}

/**
 * An implementation that allows registering child [ActivityLifecycleCallbacks] and then silencing
 * them via the [SilenceableLifecycleCallbackProxy] interface.
 *
 * We use a Singleton pattern via the [INSTANCE] field in the companion object instead of DI, as
 * this is required very early in the application lifecycle.
 */
class SilenceableIndirectLifecycleCallbackProxyImpl private constructor() :
    SilenceableLifecycleCallbackProxy,
    IndirectLifecycleCallbackProxy {

    companion object {
        val INSTANCE = SilenceableIndirectLifecycleCallbackProxyImpl()
    }

    private val callbacks = ArrayList<ActivityLifecycleCallbacks?>()

    /**
     * The life cycle callbacks are silenced if there is at least one active request to do so.
     */
    private val numSilenceTicketsActive = AtomicInteger(0)

    //
    // SilenceableLifecycleCallbackProxy
    //

    override fun acquireSilenceTicket(): LifecycleCallbackProxySilenceTicket {
        numSilenceTicketsActive.incrementAndGet()
        return LifecycleCallbackProxySilenceTicketImpl(numSilenceTicketsActive)
    }

    //
    // IndirectLifecycleCallbackProxy
    //

    override fun registerActivityLifecycleCallbacks(callback: ActivityLifecycleCallbacks?) {
        if (callback == null) return  // required due to nullable parameter from Java interface
        synchronized(callbacks) {
            callbacks.add(callback)
        }
    }

    override fun unregisterActivityLifecycleCallbacks(callback: ActivityLifecycleCallbacks?) {
        if (callback == null) return // required due to nullable parameter from Java interface
        synchronized(callbacks) {
            callbacks.remove(callback)
        }
    }

    /**
     * Caches the callbacks into an array. This is helpful (and also done in the Android API
     * implementation `Application`) to avoid keeping the lock on [callbacks] while executing
     * all callbacks.
     *
     * If there is any active silence tickets (i.e.[numSilenceTicketsActive] is non-zero), an
     * empty array is returned.
     */
    private fun collectCallbacks(): Array<ActivityLifecycleCallbacks> {
        if (numSilenceTicketsActive.get() > 0) return emptyArray()
        synchronized(callbacks) {
            if (callbacks.isEmpty()) {
                return emptyArray()
            } else {
                // the unchecked cast to `ActivityLifecycleCallbacks` instead of
                // `ActivityLifecycleCallbacks?` is fine as we check for non-null in the
                // `register...` and `unregister...` methods.
                @Suppress("UNCHECKED_CAST")
                return callbacks.toTypedArray() as Array<ActivityLifecycleCallbacks>
            }
        }
    }

    //
    // ActivityLifecycleCallbacks
    //

    override fun onActivityCreated(activity: Activity, savedInstanceState: Bundle?) {
        collectCallbacks().forEach { it.onActivityCreated(activity, savedInstanceState) }
    }

    override fun onActivityStarted(activity: Activity) {
        collectCallbacks().forEach { it.onActivityStarted(activity) }
    }

    override fun onActivityResumed(activity: Activity) {
        collectCallbacks().forEach { it.onActivityResumed(activity) }
    }

    override fun onActivityPaused(activity: Activity) {
        collectCallbacks().forEach { it.onActivityPaused(activity) }
    }

    override fun onActivityStopped(activity: Activity) {
        collectCallbacks().forEach { it.onActivityStopped(activity) }
    }

    override fun onActivitySaveInstanceState(activity: Activity, outState: Bundle) {
        collectCallbacks().forEach { it.onActivitySaveInstanceState(activity, outState) }
    }

    override fun onActivityDestroyed(activity: Activity) {
        collectCallbacks().forEach { it.onActivityDestroyed(activity) }
    }
}

/**
 * Implementation of the [LifecycleCallbackProxySilenceTicket] that references the [AtomicInteger]
 * used for counting the number of silencing requests.
 *
 * It overrides [finalize] as a safe-guard that is triggered if the app code forgot to release the
 * lock and this object is being garbage collected.
 */
internal class LifecycleCallbackProxySilenceTicketImpl(private val globalActiveTickets: AtomicInteger) :
    LifecycleCallbackProxySilenceTicket {

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
