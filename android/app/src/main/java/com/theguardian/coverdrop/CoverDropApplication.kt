package com.theguardian.coverdrop

import android.app.Application
import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.tracking.SampleLifecycleListener
import com.theguardian.coverdrop.ui.tracking.IndirectLifecycleCallbackProxy
import com.theguardian.coverdrop.ui.tracking.SilenceableIndirectLifecycleCallbackProxyImpl
import com.theguardian.coverdrop.ui.tracking.SilenceableUncaughtExceptionHandlerProxyImpl
import dagger.hilt.android.HiltAndroidApp
import javax.inject.Inject

@HiltAndroidApp
class CoverDropApplication : Application() {

    @Inject
    lateinit var coverdrop: ICoverDropLib

    @Inject
    lateinit var sampleLifecycleListener: SampleLifecycleListener

    // required before the application has fully initialized from Dagger perspective
    private val lifecycleCallbackProxy: IndirectLifecycleCallbackProxy =
        SilenceableIndirectLifecycleCallbackProxyImpl.INSTANCE

    override fun onCreate() {
        super.onCreate()
        super.registerActivityLifecycleCallbacks(lifecycleCallbackProxy)

        // Initialisation of a sample third-party library that might register lifecycle callbacks.
        // This is just for demonstration, and an actual application might register its own
        // libraries here or none at all.
        sampleLifecycleListener.onAppInit(this)

        // We initialize the silenceable exception handler as an additional protection against
        // leaking information via crash-logs. While in the CoverDropActivity, the app will simply
        // exit without any crash reporting. Implementing apps should call this as the last update
        // to the exception handler to ensure it wraps all others.
        SilenceableUncaughtExceptionHandlerProxyImpl.INSTANCE.init(Thread.getDefaultUncaughtExceptionHandler()!!)
    }

    override fun registerActivityLifecycleCallbacks(callback: ActivityLifecycleCallbacks?) {
        lifecycleCallbackProxy.registerActivityLifecycleCallbacks(callback)
    }

    override fun unregisterActivityLifecycleCallbacks(callback: ActivityLifecycleCallbacks?) {
        lifecycleCallbackProxy.unregisterActivityLifecycleCallbacks(callback)
    }
}
