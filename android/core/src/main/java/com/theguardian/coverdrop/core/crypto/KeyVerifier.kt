package com.theguardian.coverdrop.core.crypto

import androidx.annotation.VisibleForTesting
import com.goterl.lazysodium.SodiumAndroid
import com.theguardian.coverdrop.core.api.models.PublishedCoverNodeKeyHierarchy
import com.theguardian.coverdrop.core.api.models.PublishedJournalistsKeyHierarchy
import com.theguardian.coverdrop.core.api.models.PublishedKeyFamily
import com.theguardian.coverdrop.core.api.models.PublishedKeyHierarchy
import com.theguardian.coverdrop.core.api.models.PublishedKeysAndProfiles
import com.theguardian.coverdrop.core.api.models.PublishedSignedEncryptionKey
import com.theguardian.coverdrop.core.api.models.PublishedSignedSigningKey
import com.theguardian.coverdrop.core.api.models.TrustedRootSigningKey
import com.theguardian.coverdrop.core.api.models.VerifiedCoverNodeKeyHierarchy
import com.theguardian.coverdrop.core.api.models.VerifiedJournalistsKeyHierarchy
import com.theguardian.coverdrop.core.api.models.VerifiedKeyFamily
import com.theguardian.coverdrop.core.api.models.VerifiedKeyHierarchy
import com.theguardian.coverdrop.core.api.models.VerifiedKeys
import com.theguardian.coverdrop.core.api.models.VerifiedSignedEncryptionKey
import com.theguardian.coverdrop.core.api.models.VerifiedSignedSigningKey
import com.theguardian.coverdrop.core.utils.hexDecode
import java.time.Instant


internal open class KeyVerificationException(
    message: String? = null,
    cause: Exception? = null,
) : Exception(message, cause)

internal class KeyExpirationException(
    message: String? = null,
    cause: Exception? = null,
) : KeyVerificationException(message, cause)

/**
 * Indicates that the downloaded public keys contained at least one key hierarchy for which the
 * organisation key could not be verified against the list of trusted organisation keys.
 */
internal class KeyMissingTrustAnchorException(
    message: String? = null,
    cause: Exception? = null,
) : KeyVerificationException(message, cause)


/**
 * The methods of the [KeyVerifier] take [PublishedKeysAndProfiles] and its members and return verified keys.
 * All methods are pure function and external data (list of trusted keys, current time) need to be
 * passed in.
 */
internal class KeyVerifier(private val libSodium: SodiumAndroid) {

    /**
     * Verifies the entire [PublishedKeysAndProfiles] by verifying each of the included [VerifiedKeyHierarchy]
     * members as per [verifyPublishedKeyHierarchy].
     */
    internal fun verifyPublishedKeysAndProfiles(
        publishedKeysAndProfiles: PublishedKeysAndProfiles,
        trustedOrgPks: List<PublicSigningKey>,
        now: Instant,
    ): VerifiedKeys {
        val verifiedKeyHierarchies = publishedKeysAndProfiles.keys.map { keyHierarchy ->
            verifyPublishedKeyHierarchy(keyHierarchy, trustedOrgPks, now)
        }
        return VerifiedKeys(keys = verifiedKeyHierarchies)
    }

    /**
     * Verifies the entire key hierarchy of [PublishedKeysAndProfiles]. This method returns a
     * [VerifiedKeyHierarchy] that only contains the keys that verified correctly. All other items
     * will be not included. However, it will throw a [KeyVerificationException] if critical keys
     * such as the organisation key or the provision keys fail to verify.
     */
    private fun verifyPublishedKeyHierarchy(
        publishedKeys: PublishedKeyHierarchy,
        trustedOrgPks: List<PublicSigningKey>,
        now: Instant,
    ): VerifiedKeyHierarchy {
        // establish the trusted root key
        val orgPk = verifyTrustedRootKeyOrThrow(
            orgPk = publishedKeys.orgPk,
            trustedOrgPks = trustedOrgPks,
            now = now,
        )

        val verifiedCoverNodeKeyHierarchies =
            publishedKeys.coverNodesKeyHierarchy.map { coverNodeKeyHierarchy ->
                verifyPublishedCoverNodeKeyHierarchy(coverNodeKeyHierarchy, orgPk, now)
            }
        val verifiedJournalistsKeyHierarchies =
            publishedKeys.journalistsKeyHierarchy.map { journalistsKeyHierarchy ->
                verifyPublishedJournalistsKeyHierarchy(journalistsKeyHierarchy, orgPk, now)
            }

        return VerifiedKeyHierarchy(
            orgPk = orgPk,
            journalistsHierarchies = verifiedJournalistsKeyHierarchies,
            coverNodeHierarchies = verifiedCoverNodeKeyHierarchies
        )
    }

    /**
     * Verifies the [PublishedJournalistsKeyHierarchy] by checking that the provisioning key
     * verifies under the [orgPk] and that all journalist keys verify under the provisioning key.
     * The returned [VerifiedJournalistsKeyHierarchy] will only contain results as per
     * [verifyKeyFamilies].
     */
    private fun verifyPublishedCoverNodeKeyHierarchy(
        coverNodeKeyHierarchy: PublishedCoverNodeKeyHierarchy,
        orgPk: TrustedRootSigningKey,
        now: Instant,
    ): VerifiedCoverNodeKeyHierarchy {
        val provisioningPk = verifySigningKeyWithExpiryOrThrow(
            candidate = coverNodeKeyHierarchy.provisioningPk,
            parent = orgPk.pk,
            now = now
        )

        val coverNodes = coverNodeKeyHierarchy.coverNodes.mapValues { (coverNodeId, keyFamilies) ->
            verifyKeyFamilies(keyFamilies, provisioningPk, IdKeyRole.COVERNODE_ID, coverNodeId, now)
        }

        return VerifiedCoverNodeKeyHierarchy(
            provisioningPk = provisioningPk,
            coverNodes = coverNodes
        )
    }

    /**
     * Verifies the [PublishedJournalistsKeyHierarchy] by checking that the provisioning key
     * verifies under the [orgPk] and that all journalist keys verify under the provisioning key.
     * The returned [VerifiedJournalistsKeyHierarchy] will only contain results as per
     * [verifyKeyFamilies].
     */
    private fun verifyPublishedJournalistsKeyHierarchy(
        journalistsKeyHierarchy: PublishedJournalistsKeyHierarchy,
        orgPk: TrustedRootSigningKey,
        now: Instant,
    ): VerifiedJournalistsKeyHierarchy {
        val provisioningPk = verifySigningKeyWithExpiryOrThrow(
            candidate = journalistsKeyHierarchy.provisioningPk,
            parent = orgPk.pk,
            now = now
        )

        val journalists = journalistsKeyHierarchy.journalists.mapValues { (journalistId, keyFamilies) ->
            verifyKeyFamilies(keyFamilies, provisioningPk, IdKeyRole.JOURNALIST_ID, journalistId, now)
        }

        return VerifiedJournalistsKeyHierarchy(
            provisioningPk = provisioningPk,
            journalists = journalists
        )
    }

    /**
     * Verifies a list of [PublishedKeyFamily] under the given [provisioningKey]. The returned
     * list will only contain [VerifiedKeyFamily] items where the respective
     * [VerifiedKeyFamily.idPk] verified under the [provisioningKey] for the given [role] and
     * [identity] (the key under which the families were published). Also, each item in the
     * [VerifiedKeyFamily.msgPks] list will only contain [VerifiedSignedEncryptionKey] items that
     * verified under the [VerifiedKeyFamily.idPk].
     */
    @VisibleForTesting
    fun verifyKeyFamilies(
        keyFamilies: List<PublishedKeyFamily>,
        provisioningKey: VerifiedSignedSigningKey,
        role: IdKeyRole,
        identity: String,
        now: Instant,
    ): List<VerifiedKeyFamily> {
        return keyFamilies.mapNotNull { keyFamily ->
            verifyKeyFamily(keyFamily, provisioningKey, role, identity, now)
        }
    }

    /**
     * Verifies a single [PublishedKeyFamily] under the given [provisioningKey]. The returned
     * [VerifiedKeyFamily] will only contain [VerifiedSignedEncryptionKey] items that verified
     * under the [VerifiedKeyFamily.idPk]. If the [PublishedKeyFamily.idPk] does not verify under
     * the [provisioningKey], `null` is returned.
     */
    private fun verifyKeyFamily(
        keyFamily: PublishedKeyFamily,
        provisioningKey: VerifiedSignedSigningKey,
        role: IdKeyRole,
        identity: String,
        now: Instant,
    ): VerifiedKeyFamily? {
        val idPk = verifyIdSigningKeyWithExpiryOrNull(
            candidate = keyFamily.idPk,
            parent = provisioningKey.pk,
            role = role,
            identity = identity,
            now = now
        ) ?: return null

        val msgKeys = keyFamily.msgPks.mapNotNull { msgPk ->
            verifyEncryptionKeyWithExpiryOrNull(
                candidate = msgPk,
                parent = idPk.pk,
                now = now
            )
        }

        return VerifiedKeyFamily(idPk = idPk, msgPks = msgKeys)
    }


    //
    // Key-level verification methods
    //

    /**
     * Verifies an organisation key against a known list of trusted organisation keys. Returns a
     * [TrustedRootSigningKey] if there is a match and throws [KeyVerificationException]
     * otherwise. The [orgPk] is expected to be a [PublishedSignedSigningKey] that is signed by
     * the itself such that the expiry date is authenticated.
     */
    @VisibleForTesting
    fun verifyTrustedRootKeyOrThrow(
        orgPk: PublishedSignedSigningKey,
        trustedOrgPks: List<PublicSigningKey>,
        now: Instant,
    ): TrustedRootSigningKey {
        val candidatePk = PublicSigningKey(orgPk.key.hexDecode())

        for (trustedOrgPk in trustedOrgPks) {
            if (trustedOrgPk == candidatePk) {
                // We found a trust anchor that matches this candidate key: there is no other anchor
                // to verify against, so failing this signature check is fatal
                val verifiedKey = verifySigningKeyWithExpiryOrThrow(
                    candidate = orgPk,
                    parent = trustedOrgPk,
                    now = now
                )
                return TrustedRootSigningKey(pk = verifiedKey.pk)
            }
        }

        throw KeyMissingTrustAnchorException("organisation key is not trusted")
    }

    /**
     * Verifies the [candidate] signing key by checking that the expiry date has not been passed
     * and the its certificate signs the key and the not-valid-after date using the [parent] key.
     *
     * @return the [VerifiedSignedSigningKey] iff all checks pass; otherwise `null`
     */
    fun verifySigningKeyWithExpiryOrNull(
        candidate: PublishedSignedSigningKey,
        parent: PublicSigningKey,
        now: Instant,
    ): VerifiedSignedSigningKey? {
        return try {
            verifySigningKeyWithExpiryOrThrow(candidate, parent, now)
        } catch (e: KeyVerificationException) {
            null
        }
    }

    /**
     * Verifies the [candidate] signing key by checking that the expiry date has not been passed
     * and the its certificate signs the key and the not-valid-after date using the [parent] key.
     * If a `signature` is present, it must verify over the same data.
     *
     * @throws [KeyExpirationException] if the key has expired.
     * @throws [KeyVerificationException] if the key is not valid for any other reason.
     */
    @VisibleForTesting
    fun verifySigningKeyWithExpiryOrThrow(
        candidate: PublishedSignedSigningKey,
        parent: PublicSigningKey,
        now: Instant,
    ): VerifiedSignedSigningKey {
        val candidateKey = PublicSigningKey(candidate.key.hexDecode())
        return verifySigningKeyOrThrow(
            candidate = candidate,
            candidateKey = candidateKey,
            parent = parent,
            now = now,
            signatureData = SigningKeyCertificateData.from(
                key = candidateKey,
                notValidAfter = candidate.notValidAfter
            ),
        )
    }

    /**
     * Like [verifySigningKeyWithExpiryOrNull], but for identity keys where a present `signature`
     * must bind the key to the given [role] and [identity].
     *
     * @return the [VerifiedSignedSigningKey] iff all checks pass; otherwise `null`
     */
    fun verifyIdSigningKeyWithExpiryOrNull(
        candidate: PublishedSignedSigningKey,
        parent: PublicSigningKey,
        role: IdKeyRole,
        identity: String,
        now: Instant,
    ): VerifiedSignedSigningKey? {
        return try {
            verifyIdSigningKeyWithExpiryOrThrow(candidate, parent, role, identity, now)
        } catch (e: KeyVerificationException) {
            null
        }
    }

    /**
     * Like [verifySigningKeyWithExpiryOrThrow], but for identity keys where a present `signature`
     * must bind the key to the given [role] and [identity].
     *
     * @throws [KeyExpirationException] if the key has expired.
     * @throws [KeyVerificationException] if the key is not valid for any other reason.
     */
    @VisibleForTesting
    fun verifyIdSigningKeyWithExpiryOrThrow(
        candidate: PublishedSignedSigningKey,
        parent: PublicSigningKey,
        role: IdKeyRole,
        identity: String,
        now: Instant,
    ): VerifiedSignedSigningKey {
        val candidateKey = PublicSigningKey(candidate.key.hexDecode())
        return verifySigningKeyOrThrow(
            candidate = candidate,
            candidateKey = candidateKey,
            parent = parent,
            now = now,
            signatureData = IdKeyCertificateData.from(
                key = candidateKey,
                notValidAfter = candidate.notValidAfter,
                role = role,
                identity = identity
            ),
        )
    }

    private fun verifySigningKeyOrThrow(
        candidate: PublishedSignedSigningKey,
        candidateKey: PublicSigningKey,
        parent: PublicSigningKey,
        now: Instant,
        signatureData: Signable,
    ): VerifiedSignedSigningKey {
        val candidateNotValidAfter = candidate.notValidAfter

        // CHECK 1: verify that the not-valid-after date has not passed
        if (now.isAfter(candidateNotValidAfter)) {
            throw KeyExpirationException("failed to verify signing key: expired on $candidateNotValidAfter")
        }

        // CHECK 2: verify the signature (if present)
        candidate.signature?.let { signature ->
            try {
                Signature.verifyOrThrow(
                    libSodium = libSodium,
                    signingPk = parent,
                    data = signatureData,
                    signature = Signature(signature.hexDecode())
                )
            } catch (e: Exception) {
                throw KeyVerificationException("failed to verify signing key signature: ${e.message}", e)
            }
        }

        // CHECK 3: verify that the certificate is valid
        // TODO (https://github.com/guardian/coverdrop-internal/issues/4200) remove once all keys carry a signature
        try {
            Signature.verifyOrThrow(
                libSodium = libSodium,
                signingPk = parent,
                data = SigningKeyCertificateData.from(
                    key = candidateKey,
                    notValidAfter = candidateNotValidAfter
                ),
                signature = Signature(candidate.certificate.hexDecode())
            )
            return VerifiedSignedSigningKey(pk = candidateKey)
        } catch (e: Exception) {
            throw KeyVerificationException("failed to verify signing key: ${e.message}", e)
        }
    }

    /**
     * Verifies the [candidate] encryption key by checking that the expiry date has not been passed
     * and the its certificate signs the key and the not-valid-after date using the [parent] key.
     *
     * @return the [VerifiedSignedEncryptionKey] iff all checks pass; otherwise `null`
     */
    fun verifyEncryptionKeyWithExpiryOrNull(
        candidate: PublishedSignedEncryptionKey,
        parent: PublicSigningKey,
        now: Instant,
    ): VerifiedSignedEncryptionKey? {
        return try {
            verifyEncryptionKeyWithExpiryOrThrow(candidate, parent, now)
        } catch (e: KeyVerificationException) {
            null
        }
    }

    /**
     * Verifies the [candidate] encryption key by checking that the expiry date has not been passed
     * and the its certificate signs the key and the not-valid-after date using the [parent] key.
     *
     * @throws [KeyExpirationException] if the key has expired.
     * @throws [KeyVerificationException] if the key is not valid for any other reason.
     */
    @VisibleForTesting
    fun verifyEncryptionKeyWithExpiryOrThrow(
        candidate: PublishedSignedEncryptionKey,
        parent: PublicSigningKey,
        now: Instant,
    ): VerifiedSignedEncryptionKey {
        val candidateKey = PublicEncryptionKey(candidate.key.hexDecode())
        val candidateNotValidAfter = candidate.notValidAfter
        val candidateCertificateData = EncryptionKeyWithExpiryCertificateData.from(
            key = candidateKey,
            notValidAfter = candidateNotValidAfter
        )

        // CHECK 1: verify that the not-valid-after date has not passed
        if (now.isAfter(candidateNotValidAfter)) {
            throw KeyExpirationException("failed to verify encryption key: expired on $candidateNotValidAfter")
        }

        // CHECK 2: verify the signature (if present)
        candidate.signature?.let { signature ->
            try {
                Signature.verifyOrThrow(
                    libSodium = libSodium,
                    signingPk = parent,
                    data = candidateCertificateData,
                    signature = Signature(signature.hexDecode())
                )
            } catch (e: Exception) {
                throw KeyVerificationException("failed to verify encryption key signature: ${e.message}", e)
            }
        }

        // CHECK 3: verify that the certificate is valid
        try {
            Signature.verifyOrThrow(
                libSodium = libSodium,
                signingPk = parent,
                data = candidateCertificateData,
                signature = Signature(candidate.certificate.hexDecode())
            )
            return VerifiedSignedEncryptionKey(
                pk = candidateKey,
                notValidAfter = candidateNotValidAfter
            )
        } catch (e: Exception) {
            throw KeyVerificationException("failed to verify encryption key: ${e.message}", e)
        }
    }
}
