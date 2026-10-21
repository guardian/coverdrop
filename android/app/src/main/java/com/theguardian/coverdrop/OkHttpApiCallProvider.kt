package com.theguardian.coverdrop

import com.theguardian.coverdrop.core.api.ApiCallProviderException
import com.theguardian.coverdrop.core.api.ApiConfiguration
import com.theguardian.coverdrop.core.api.IApiCallProvider
import okhttp3.CertificatePinner.Companion.sha256Hash
import okhttp3.HttpUrl
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.dnsoverhttps.DnsOverHttps
import okio.ByteString
import okio.ByteString.Companion.decodeHex
import java.net.InetAddress
import java.security.cert.X509Certificate
import javax.net.ssl.HostnameVerifier
import javax.net.ssl.SSLSession

/**
 * Sample implementation of [IApiCallProvider] using [OkHttpClient].
 */
class OkHttpApiCallProvider : IApiCallProvider {

    private val httpClient = createOkHttpClient()

    private fun createOkHttpClient(): OkHttpClient {
        val client: OkHttpClient = OkHttpClient.Builder()
            .hostnameVerifier(ForbiddenCertificatesHostnameVerifier())
            .build()

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

/**
 * Confirm the server's certificates are not on a specific denylist.
 *
 * Use this if the platform's trusted certificates contain certificates later found untrustworthy.
 *
 * Source: https://publicobject.com/2026/10/20/mitigating-bad-roots-in-okhttp/
 */
private class ForbiddenCertificatesHostnameVerifier(
    private val forbiddenSpkiSha256s: Set<ByteString> = CVE_2026_111943,
    private val delegate: HostnameVerifier = OkHttpClient().hostnameVerifier,
) : HostnameVerifier {

    override fun verify(hostname: String, session: SSLSession): Boolean {
        val delegateResult = delegate.verify(hostname, session)
        return delegateResult && !hasForbiddenPeerCertificates(session)
    }

    private fun hasForbiddenPeerCertificates(session: SSLSession): Boolean =
        session.peerCertificates.any { certificate ->
            val sha256Hash = (certificate as X509Certificate).sha256Hash()
            sha256Hash in forbiddenSpkiSha256s
        }

    companion object {
        /** https://groups.google.com/a/mozilla.org/g/dev-security-policy/c/0R3XLJL6FuE/m/0XofdebrAAAJ?pli=1 */
        val CVE_2026_111943 = setOf(
            "43bffa19d6ef651451f6841e019931e96f6f2eaeff28ab4b6f1ce4576de2717b".decodeHex(),
            "49dd353dc2b919a3770645117d0e60c16115e7d1bc259eefe734b00ea434fe5f".decodeHex(),
            "5c0660a2b164910d8ff589c18e14552e9e0ec237dcefe2cb736d41a4811f81a0".decodeHex(),
            "63ecefee431d49523bc1136c4e72d0206a477086caf606cd5cf025686c109cf8".decodeHex(),
            "92c063d67e8f14db71d288e11266ffdaf69cbf115f7bab14f3bc2f68d25e0e30".decodeHex(),
            "e0254c7fcdc265e2f0f8a428dabb75ab4ae039b4872d62f5ee57f0877d5a8a41".decodeHex(),
            "ebed6475c2c2d80fcb199f3fe8dd6e6b3ce60b77eb15d4e42c3dec0c6f5b5105".decodeHex(),
            "ec4ffa1ad0ac88081314b7aaffef6a91f97cd57291bb5ba686c5415f5eb4f6b6".decodeHex(),
        )
    }
}