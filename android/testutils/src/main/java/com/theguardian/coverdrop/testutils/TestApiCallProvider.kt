package com.theguardian.coverdrop.testutils

import android.annotation.SuppressLint
import com.theguardian.coverdrop.core.api.ApiConfiguration
import com.theguardian.coverdrop.core.api.IApiCallProvider
import com.theguardian.coverdrop.core.api.createGsonInstance
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDropsList

class TestApiCallProvider(private val testVectors: IntegrationTestVectors) : IApiCallProvider {

    /**
     * All post requests that would have been made to the API as pairs of the `path` and `json`
     * content.
     */
    private val loggedPostRequests = mutableListOf<Pair<String, String>>()

    private var publishedKeysFileName: String? = null
    private var userDeadDropsFileName: String? = null

    override suspend fun getJsonApi(
        apiConfiguration: ApiConfiguration,
        path: String,
        queryParameters: List<Pair<String, String>>?,
    ): String {
        return when (path) {
            "/v1/public-keys" -> testVectors.readJson("published_keys", publishedKeysFileName)
            "/v1/user/dead-drops" -> getUserDeadDrops(queryParameters!!.toMap())
            "/v1/status" -> testVectors.readJson("system_status")
            else -> TODO("not yet implemented: $path")
        }
    }

    override suspend fun postJsonMessaging(
        apiConfiguration: ApiConfiguration,
        path: String,
        json: String,
    ) {
        // without a mocked server, we simply record all post requests
        loggedPostRequests.add(path to json)
    }

    fun clearLoggedPostRequests() = loggedPostRequests.clear()

    fun getLoggedPostRequests() = loggedPostRequests.toList()

    fun getLoggedPostRequestEndpoints() = loggedPostRequests.map { it.first }

    /**
     * Call to overwrite the file that is returned by the API for `/v1/user/dead-drops`. Otherwise,
     * the first file in the folder (usually starting with "001_") is returned.
     */
    fun setUserDeadDropsFileName(fileName: String?) {
        userDeadDropsFileName = fileName
    }

    /**
     * Call to overwrite the file that is returned by the API for `/v1/user/public-keys`.
     * Otherwise, the first file in the folder (usually starting with "001_") is returned.
     */
    fun setPublicKeysFileName(fileName: String?) {
        publishedKeysFileName = fileName
    }

    /**
     * Simulate back-end logic returning only dead-drops past a certain ID
     */
    @SuppressLint("VisibleForTests")
    private fun getUserDeadDrops(queryParameters: Map<String, String>): String {
        val gson = createGsonInstance()

        val inputDeadDropsJson = testVectors.readJson("user_dead_drops", userDeadDropsFileName)
        val inputDeadDrops = gson.fromJson(
            inputDeadDropsJson,
            PublishedJournalistToUserDeadDropsList::class.java
        )

        val idsGreaterThan = queryParameters["ids_greater_than"]!!.toInt()
        val filteredVectors = PublishedJournalistToUserDeadDropsList(
            deadDrops = inputDeadDrops.deadDrops.filter { it.id > idsGreaterThan }
        )

        return gson.toJson(filteredVectors)
    }

    companion object {
        fun createTestApiConfiguration() = ApiConfiguration(
            apiBaseUrl = "http://example.org",
            messagingBaseUrl = "http://example.org"
        )
    }
}
