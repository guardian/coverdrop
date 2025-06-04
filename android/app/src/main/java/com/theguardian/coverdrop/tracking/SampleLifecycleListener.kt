package com.theguardian.coverdrop.tracking

import android.app.Activity
import android.app.Application
import android.os.Bundle
import android.util.Log

/**
 * This is a sample lifecycle listener. We use it in our reference app to simulate a third-party
 * library that hooks into the app lifecycle and might therefore leak usage information.
 */
class SampleLifecycleListener : Application.ActivityLifecycleCallbacks {

    fun onAppInit(application: Application) {
        application.registerActivityLifecycleCallbacks(this)
    }

    override fun onActivityCreated(activity: Activity, savedInstanceState: Bundle?) {
        Log.i("SampleLifecycleListener", "onActivityCreated ${activity.localClassName}")
    }

    override fun onActivityStarted(activity: Activity) {
        Log.i("SampleLifecycleListener", "onActivityStarted ${activity.localClassName}")
    }

    override fun onActivityResumed(activity: Activity) {
        Log.i("SampleLifecycleListener", "onActivityResumed ${activity.localClassName}")
    }

    override fun onActivityPaused(activity: Activity) {
        Log.i("SampleLifecycleListener", "onActivityPaused ${activity.localClassName}")
    }

    override fun onActivityStopped(activity: Activity) {
        Log.i("SampleLifecycleListener", "onActivityStopped ${activity.localClassName}")
    }

    override fun onActivitySaveInstanceState(activity: Activity, outState: Bundle) {
        Log.i("SampleLifecycleListener", "onActivitySaveInstanceState ${activity.localClassName}")
    }

    override fun onActivityDestroyed(activity: Activity) {
        Log.i("SampleLifecycleListener", "onActivityDestroyed ${activity.localClassName}")
    }
}
