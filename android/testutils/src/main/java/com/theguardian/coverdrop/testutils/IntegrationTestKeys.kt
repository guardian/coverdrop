package com.theguardian.coverdrop.testutils

import android.annotation.SuppressLint
import android.content.Context
import androidx.annotation.Keep
import com.google.gson.annotations.SerializedName
import com.theguardian.coverdrop.core.api.createGsonInstance
import com.theguardian.coverdrop.core.api.models.PublicKey
import com.theguardian.coverdrop.core.api.models.PublishedSignedSigningKey
import com.theguardian.coverdrop.core.crypto.PublicSigningKey
import java.io.File
import java.time.Instant


class IntegrationTestKeys(
    private val context: Context,
    keysPath: String = "integration-test-keys",
) {
    private val basePathFile = File(keysPath)

    @SuppressLint("VisibleForTests")
    private val gson = createGsonInstance()

    fun getNow(): Instant {
        val instantString = readString("keys_generated_at.txt")
        return Instant.parse(instantString)
    }

    fun getOrganisationKey(): PublishedSignedSigningKey {
        val organisationKeyJson = readString(findSingleFile("organization-", ".pub.json"))
        return gson.fromJson(organisationKeyJson, PublishedSignedSigningKey::class.java)
    }

    fun getUserKeyPair(): TestUserKeyPair {
        val userKeyPairJson = readString(findSingleFile("user-", ".keypair.json"))
        return gson.fromJson(userKeyPairJson, TestUserKeyPair::class.java)
    }

    fun getTrustedOrganisationKeys(): List<PublicSigningKey> {
        return listOf(PublicSigningKey.fromHexEncodedString(getOrganisationKey().key))
    }

    private fun findSingleFile(prefix: String, suffix: String): String {
        return context.assets.list(basePathFile.path)!!
            .single { it.startsWith(prefix) && it.endsWith(suffix) }
    }

    private fun readString(filename: String): String {
        val path = File(basePathFile, filename)
        return context.assets.open(path.path).reader().readText().trim()
    }
}

@Keep // required to survive R8
data class TestUserKeyPair(
    @SerializedName("secret_key")
    val secretKey: String,

    @SerializedName("public_key")
    val publicKey: PublicKey,
)
