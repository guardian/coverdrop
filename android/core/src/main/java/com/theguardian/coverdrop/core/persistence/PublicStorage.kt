package com.theguardian.coverdrop.core.persistence

import android.annotation.SuppressLint
import android.content.Context
import android.content.SharedPreferences
import androidx.annotation.VisibleForTesting
import androidx.core.content.edit
import com.google.gson.Gson
import com.theguardian.coverdrop.core.api.createGsonInstance
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDropsList
import com.theguardian.coverdrop.core.api.models.PublishedKeysAndProfiles
import com.theguardian.coverdrop.core.api.models.PublishedStatusEvent
import com.theguardian.coverdrop.core.persistence.SharedPreferencesKeys.PREF_KEY_BACKGROUND_WORK_LAST_RUN_TIMESTAMP
import com.theguardian.coverdrop.core.persistence.SharedPreferencesKeys.PREF_KEY_BACKGROUND_WORK_LAST_TRIGGERED_TIMESTAMP
import com.theguardian.coverdrop.core.persistence.SharedPreferencesKeys.PREF_KEY_BACKGROUND_WORK_PENDING
import com.theguardian.coverdrop.core.persistence.SharedPreferencesKeys.PREF_KEY_PUBLISHED_DEAD_DROPS_LAST_UPDATED
import com.theguardian.coverdrop.core.persistence.SharedPreferencesKeys.PREF_KEY_PUBLISHED_KEYS_LAST_UPDATED
import com.theguardian.coverdrop.core.persistence.SharedPreferencesKeys.PREF_KEY_SE_AVAILABLE
import com.theguardian.coverdrop.core.persistence.SharedPreferencesKeys.PREF_KEY_STATUS_EVENT_LAST_UPDATED
import java.io.FileNotFoundException
import java.time.Instant

private const val SHARED_PREF_FILE_NAME = "coverdrop_shared_prefs"

enum class SharedPreferencesKeys(val key: String) {
    PREF_KEY_BACKGROUND_WORK_PENDING("background_work_pending"),
    PREF_KEY_BACKGROUND_WORK_LAST_RUN_TIMESTAMP("background_work_last_run_timestamp"),
    PREF_KEY_BACKGROUND_WORK_LAST_TRIGGERED_TIMESTAMP("background_work_last_triggered_timestamp"),
    PREF_KEY_SE_AVAILABLE("se_available"),
    PREF_KEY_STATUS_EVENT_LAST_UPDATED("status_event_last_update"),
    PREF_KEY_PUBLISHED_KEYS_LAST_UPDATED("published_keys_last_update"),
    PREF_KEY_PUBLISHED_DEAD_DROPS_LAST_UPDATED("published_dead_drops_last_update"),
}

/**
 * Persistence for all data that is not stored encrypted. This is primarily for caching responses
 * from the API and related metadata.
 */
internal class PublicStorage(
    private val context: Context,
    private val fileManager: CoverDropFileManager,
) {
    @SuppressLint("VisibleForTests")
    private val gson: Gson = createGsonInstance()

    fun initialize() {
        fileManager.initialize()
    }

    @SuppressLint("ApplySharedPref")
    @VisibleForTesting
    internal fun deleteAll() {
        // delete files
        for (file in CoverDropFiles.entries) {
            fileManager.delete(file)
        }

        // delete shared preferences; intentional `commit` instead of `apply` to keep it synchronous
        getSharedPreferences().edit(commit = true) {
            for (sharedPref in SharedPreferencesKeys.entries) {
                remove(sharedPref.key)
            }
        }
    }

    //
    // ApiResponseCache: StatusEvent
    //

    fun writePublishedStatusEvent(publishedStatusEventAndProfiles: PublishedStatusEvent) {
        val json = gson.toJson(publishedStatusEventAndProfiles)
        fileManager.write(CoverDropFiles.STATUS_EVENT_V1, json.encodeToByteArray())
    }

    /**
     * Returns null if no published statusEvent are stored.
     */
    fun readPublishedStatusEvent(): PublishedStatusEvent? {
        try {
            val json = fileManager.read(CoverDropFiles.STATUS_EVENT_V1).decodeToString()
            return gson.fromJson(json, PublishedStatusEvent::class.java)
        } catch (e: FileNotFoundException) {
            return null
        }
    }

    fun hasPublishedStatusEvent() = fileManager.exists(CoverDropFiles.STATUS_EVENT_V1)

    fun writePublishedStatusEventLastUpdate(instant: Instant) {
        getSharedPreferences().edit {
            putLong(PREF_KEY_STATUS_EVENT_LAST_UPDATED.key, instant.toEpochMilli())
        }
    }

    fun readPublishedStatusEventLastUpdate(): Instant? {
        val ts = getSharedPreferences().getLongOrNull(PREF_KEY_STATUS_EVENT_LAST_UPDATED)
        return ts?.run { Instant.ofEpochMilli(ts) }
    }

    //
    // ApiResponseCache: PublishedKeys
    //

    fun writePublishedKeys(publishedKeysAndProfiles: PublishedKeysAndProfiles) {
        val json = gson.toJson(publishedKeysAndProfiles)
        fileManager.write(CoverDropFiles.PUBLISHED_KEYS_V1, json.encodeToByteArray())
    }

    /**
     * Returns null if no published keys are stored.
     */
    fun readPublishedKeys(): PublishedKeysAndProfiles? {
        try {
            val json = fileManager.read(CoverDropFiles.PUBLISHED_KEYS_V1).decodeToString()
            return gson.fromJson(json, PublishedKeysAndProfiles::class.java)
        } catch (e: FileNotFoundException) {
            return null
        }
    }

    fun hasPublishedKeys() = fileManager.exists(CoverDropFiles.PUBLISHED_KEYS_V1)

    fun writePublishedKeysLastUpdate(instant: Instant) {
        getSharedPreferences().edit {
            putLong(PREF_KEY_PUBLISHED_KEYS_LAST_UPDATED.key, instant.toEpochMilli())
        }
    }

    fun readPublishedKeysLastUpdate(): Instant? {
        val ts = getSharedPreferences().getLongOrNull(PREF_KEY_PUBLISHED_KEYS_LAST_UPDATED)
        return ts?.run { Instant.ofEpochMilli(ts) }
    }

    //
    // ApiResponseCache: PublishedDeadDrops
    //

    fun writeDeadDrops(deadDrops: PublishedJournalistToUserDeadDropsList) {
        val json = gson.toJson(deadDrops)
        fileManager.write(CoverDropFiles.DEAD_DROPS_V1, json.encodeToByteArray())
    }

    /**
     * Returns an empty [PublishedJournalistToUserDeadDropsList] if no dead drops are stored.
     */
    @SuppressLint("VisibleForTests")
    fun readDeadDrops(): PublishedJournalistToUserDeadDropsList {
        try {
            val json = fileManager.read(CoverDropFiles.DEAD_DROPS_V1).decodeToString()
            return gson.fromJson(json, PublishedJournalistToUserDeadDropsList::class.java)
        } catch (e: FileNotFoundException) {
            return PublishedJournalistToUserDeadDropsList(deadDrops = emptyList())
        }
    }

    fun hasDeadDrops() = fileManager.exists(CoverDropFiles.DEAD_DROPS_V1)

    fun writePublishedDeadDropsUpdate(instant: Instant) {
        getSharedPreferences().edit {
            putLong(PREF_KEY_PUBLISHED_DEAD_DROPS_LAST_UPDATED.key, instant.toEpochMilli())
        }
    }

    fun readPublishedDeadDropsLastUpdate(): Instant? {
        val ts = getSharedPreferences().getLongOrNull(PREF_KEY_PUBLISHED_DEAD_DROPS_LAST_UPDATED)
        return ts?.run { Instant.ofEpochMilli(ts) }
    }

    //
    // PrivateSendingQueue persistence
    //

    fun writePrivateSendingQueueBytes(privateSendingQueueBytes: ByteArray) {
        fileManager.write(CoverDropFiles.PRIVATE_SENDING_QUEUE_V2, privateSendingQueueBytes)
    }

    fun readPrivateSendingQueueBytes(): ByteArray? {
        return try {
            return fileManager.read(CoverDropFiles.PRIVATE_SENDING_QUEUE_V2)
        } catch (e: FileNotFoundException) {
            null
        }
    }

    //
    // BackgroundManager state
    //

    fun writeBackgroundWorkPending(pending: Boolean) {
        getSharedPreferences().edit {
            putBoolean(PREF_KEY_BACKGROUND_WORK_PENDING.key, pending)
        }
    }

    fun readBackgroundWorkPending() =
        getSharedPreferences().getBoolean(PREF_KEY_BACKGROUND_WORK_PENDING.key, false)

    fun writeBackgroundJobLastRun(instant: Instant) {
        getSharedPreferences().edit {
            putLong(PREF_KEY_BACKGROUND_WORK_LAST_RUN_TIMESTAMP.key, instant.toEpochMilli())
        }
    }

    /**
     * Returns an instant of when the background job has successfully run last or `null` if no such
     * timestamp has been stored yet.
     */
    fun readBackgroundJobLastRun(): Instant? {
        val ts = getSharedPreferences().getLongOrNull(PREF_KEY_BACKGROUND_WORK_LAST_RUN_TIMESTAMP)
        return ts?.run { Instant.ofEpochMilli(ts) }
    }


    fun writeBackgroundJobLastTriggered() {
        getSharedPreferences().edit {
            putLong(
                PREF_KEY_BACKGROUND_WORK_LAST_TRIGGERED_TIMESTAMP.key,
                Instant.now().toEpochMilli()
            )
        }
    }

    /**
     * Returns an instant of when the background job has last been triggered or `null` if no such
     * timestamp has been stored yet. Not every trigger results in a successful run.
     *
     * @see readBackgroundJobLastRun
     */
    fun readBackgroundJobLastTriggered(): Instant? {
        val ts = getSharedPreferences().getLongOrNull(
            PREF_KEY_BACKGROUND_WORK_LAST_TRIGGERED_TIMESTAMP
        )
        return ts?.run { Instant.ofEpochMilli(ts) }
    }

    //
    // SecureElementAvailabilityCache
    //

    @SuppressLint("ApplySharedPref")
    fun writeSeAvailability(isAvailable: Boolean) {
        // intentional `commit` instead of `apply` to keep it synchronous
        getSharedPreferences().edit(commit = true) {
            putBoolean(PREF_KEY_SE_AVAILABLE.key, isAvailable)
        }
    }

    /**
     * Returns cached SE availability if previously stored or `null` if none has been stored yet.
     */
    fun readSeAvailability() = getSharedPreferences().getBooleanOrNull(PREF_KEY_SE_AVAILABLE)

    //
    // Helpers
    //

    private fun getSharedPreferences() = context.getSharedPreferences(
        SHARED_PREF_FILE_NAME,
        Context.MODE_PRIVATE,
    )
}

private fun SharedPreferences.getBooleanOrNull(sharedPrefKey: SharedPreferencesKeys): Boolean? {
    if (!contains(sharedPrefKey.key)) {
        return null
    }
    return getBoolean(
        /* key = */ sharedPrefKey.key,
        /* defValue = */ false // `defValue` is not used since key exists
    )
}

private fun SharedPreferences.getLongOrNull(sharedPrefKey: SharedPreferencesKeys): Long? {
    if (!contains(sharedPrefKey.key)) {
        return null
    }
    return getLong(
        /* key = */ sharedPrefKey.key,
        /* defValue = */ 0L // `defValue` is not used since key exists
    )
}
