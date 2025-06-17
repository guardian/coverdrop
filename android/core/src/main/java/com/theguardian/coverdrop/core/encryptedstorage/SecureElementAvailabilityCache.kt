package com.theguardian.coverdrop.core.encryptedstorage

import android.annotation.SuppressLint
import android.content.Context
import android.content.pm.PackageManager
import com.theguardian.coverdrop.core.persistence.PublicStorage
import kotlin.properties.Delegates

/**
 * Caches the availability of the SE on first app start. This ensures that we always pick the same
 * parameters (for instance passphrase length) for subsequent sessions.
 */
@SuppressLint("ApplySharedPref")
internal open class SecureElementAvailabilityCache(
    context: Context,
    publicStorage: PublicStorage,
) {
    private var cachedIsAvailable by Delegates.notNull<Boolean>()

    init {
        val maybeIsAvailable = publicStorage.readSeAvailability()
        if (maybeIsAvailable == null) {
            publicStorage.writeSeAvailability(internalCheckIsAvailable(context))
        }

        // now guaranteed to be non-null
        cachedIsAvailable = publicStorage.readSeAvailability()!!
    }

    /**
     * Returns true if a secure element is available. This method is guaranteed to always return
     * the same value even after app restarts and OS updates.
     */
    fun isAvailable(): Boolean = cachedIsAvailable

    private fun internalCheckIsAvailable(context: Context): Boolean {
        return context.packageManager.hasSystemFeature(PackageManager.FEATURE_STRONGBOX_KEYSTORE)
    }
}

