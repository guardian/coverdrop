package com.theguardian.coverdrop

import com.theguardian.coverdrop.core.api.ApiCallProviderException
import com.theguardian.coverdrop.core.api.ApiConfiguration
import com.theguardian.coverdrop.core.api.IApiCallProvider
import okhttp3.HttpUrl
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.dnsoverhttps.DnsOverHttps
import java.net.InetAddress

/**
 * Sample implementation of [IApiCallProvider] using [OkHttpClient].
 */
class OkHttpApiCallProvider : IApiCallProvider {

    private val httpClient = createOkHttpClient()

    private fun createOkHttpClient(): OkHttpClient {
        val client: OkHttpClient = OkHttpClient.Builder().build()

        val dns: DnsOverHttps = DnsOverHttps.Builder().client(client)
            .url("https://cloudflare-dns.com/dns-query".toHttpUrl())
            .bootstrapDnsHosts(
                InetAddress.getByName("1.1.1.1"), // Cloudfare
                InetAddress.getByName("8.8.4.4"), // Google
                InetAddress.getByName("8.8.8.8"), // Google
            )
            .build()

        return client.newBuilder().dns(dns).build()
    }

    override suspend fun getJsonApi(
        apiConfiguration: ApiConfiguration,
        path: String,
        queryParameters: List<Pair<String, String>>?,
    ): String {
        val url = buildFullUrl(
            baseUrlString = apiConfiguration.apiBaseUrl,
            pathString = path,
            queryParameters = queryParameters,
        )

        try {
            val request = Request.Builder()
                .url(url)
                .get()
                .build()

            val response = httpClient.newCall(request).execute()
            check(response.isSuccessful)

            return response.body!!.string()
        } catch (e: Exception) {
            throw ApiCallProviderException("getJson($path) failed: ${e.message}", e)
        }
    }

    override suspend fun postJsonMessaging(
        apiConfiguration: ApiConfiguration,
        path: String,
        json: String,
    ) {
        val url = buildFullUrl(
            baseUrlString = apiConfiguration.messagingBaseUrl,
            pathString = path,
        )

        try {
            val requestBody = json.toRequestBody("application/json".toMediaType())
            val request = Request.Builder()
                .url(url)
                .post(requestBody)
                .build()

            val response = httpClient.newCall(request).execute()
            check(response.isSuccessful)
        } catch (e: Exception) {
            throw ApiCallProviderException("getJson($path) failed: ${e.message}", e)
        }
    }

    private fun buildFullUrl(
        baseUrlString: String,
        pathString: String,
        queryParameters: List<Pair<String, String>>? = null,
    ): HttpUrl {
        val url = try {
            val combinedUrl = "$baseUrlString$pathString"
            combinedUrl.toHttpUrl()
        } catch (e: IllegalAccessException) {
            throw ApiCallProviderException("Failed parsing $baseUrlString", cause = e)
        }

        return if (queryParameters == null) {
            url
        } else {
            val builder = url.newBuilder()
            queryParameters.forEach { builder.addQueryParameter(it.first, it.second) }
            builder.build()
        }
    }
}
