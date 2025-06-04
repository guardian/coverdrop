package com.theguardian.coverdrop.core.api

import android.annotation.SuppressLint
import androidx.annotation.VisibleForTesting
import com.google.gson.*
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDropsList
import com.theguardian.coverdrop.core.api.models.PublishedKeysAndProfiles
import com.theguardian.coverdrop.core.api.models.PublishedStatusEvent
import com.theguardian.coverdrop.core.api.models.UserMessage
import java.lang.reflect.Type
import java.time.Instant

internal interface IApiJsonAdapter {
    fun parsePublishedStatusEvent(json: String): PublishedStatusEvent
    fun parsePublishedPublicKeys(json: String): PublishedKeysAndProfiles
    fun parsePublishedDeadDrops(json: String): PublishedJournalistToUserDeadDropsList
    fun jsonifyUserMessage(message: UserMessage): String
}

internal class GsonApiJsonAdapter : IApiJsonAdapter {
    private val gson = createGsonInstance()

    override fun parsePublishedStatusEvent(json: String): PublishedStatusEvent {
        return gson.fromJson(json, PublishedStatusEvent::class.java)
    }

    override fun parsePublishedPublicKeys(json: String): PublishedKeysAndProfiles {
        return gson.fromJson(json, PublishedKeysAndProfiles::class.java)
    }

    @SuppressLint("VisibleForTests")
    override fun parsePublishedDeadDrops(json: String): PublishedJournalistToUserDeadDropsList {
        return gson.fromJson(json, PublishedJournalistToUserDeadDropsList::class.java)
    }

    override fun jsonifyUserMessage(message: UserMessage): String {
        // we just send the string to match the `transparent` serde flag in the Rust code
        return gson.toJson(message.data)
    }
}

/**
 * Creates the default GSON instance. Note that we do not call `setLenient()` which makes the
 * parser to strictly only accept JSON compliant with RFC4627.
 */
@VisibleForTesting
fun createGsonInstance(): Gson = GsonBuilder()
    .registerTypeAdapter(Instant::class.java, JsonInstantTypeAdapter())
    .create()

private class JsonInstantTypeAdapter : JsonDeserializer<Instant>, JsonSerializer<Instant> {

    @Throws(JsonParseException::class)
    override fun deserialize(
        json: JsonElement,
        type: Type,
        context: JsonDeserializationContext,
    ): Instant {
        return Instant.parse(json.asString)
    }

    override fun serialize(
        src: Instant?,
        typeOfSrc: Type?,
        context: JsonSerializationContext?,
    ): JsonElement {
        return JsonPrimitive(src?.toString())
    }
}
