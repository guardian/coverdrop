package com.theguardian.coverdrop.core.crypto

import androidx.annotation.VisibleForTesting
import com.goterl.lazysodium.SodiumAndroid
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDrop
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDropsList
import com.theguardian.coverdrop.core.api.models.VerifiedCoverNodeKeyHierarchy
import com.theguardian.coverdrop.core.api.models.VerifiedDeadDrop
import com.theguardian.coverdrop.core.api.models.VerifiedDeadDrops
import com.theguardian.coverdrop.core.api.models.VerifiedSignedSigningKey
import com.theguardian.coverdrop.core.api.models.allCoverNodeSigningKeys
import com.theguardian.coverdrop.core.generated.JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN
import com.theguardian.coverdrop.core.utils.base64Decode
import com.theguardian.coverdrop.core.utils.hexDecode

/**
 * Behaviour of the [DeadDropParser] for dead drops that fail to verify. Failure can occur for
 * malicious (e.g. tampered data) or benign (e.g. missing signing keys) reasons.
 */
internal enum class VerificationFailureBehaviour {
    /**
     * If there is any dead-drop that does not verify, the parser will throw a
     * [DeadDropVerificationException]. This is typically the behaviour we want for tests to
     * ensure correctness of the verification routines.
     */
    THROW,

    /**
     * If there is any dead-drop that does not verify, the parser will ignore it and continue with
     * the next dead-drop. This is typically the behaviour we want for production code to ensure
     * that we can still process the verified dead-drops even if some of them fail to verify.
     *
     * A typical case is that there are dead-drops for which the signing keys are no longer being
     * published.
     */
    DROP;

    fun onFailure(): VerifiedDeadDrop? {
        when (this) {
            THROW -> throw DeadDropVerificationException("failed to verify dead drop")
            DROP -> return null
        }
    }
}

internal class DeadDropVerificationException(
    message: String? = null,
    cause: Exception? = null,
) : Exception(message, cause)


/**
 * The methods of the [DeadDropParser] take a [PublishedJournalistToUserDeadDropsList] or its
 * members and return [VerifiedDeadDrop] objects after verifying the signatures.
 */
internal class DeadDropParser(
    private val libSodium: SodiumAndroid,
    private val verificationFailureBehaviour: VerificationFailureBehaviour,
) {

    /**
     * Verifies a [PublishedJournalistToUserDeadDropsList] by checking that its certificate using the
     * available [coverNodeKeyHierarchies] candidates.
     */
    fun verifyAndParseDeadDropsList(
        candidate: PublishedJournalistToUserDeadDropsList,
        coverNodeKeyHierarchies: List<VerifiedCoverNodeKeyHierarchy>,
    ): VerifiedDeadDrops {
        return candidate.deadDrops.mapNotNull {
            verifyAndParseDeadDrop(
                it,
                coverNodeKeyHierarchies
            )
        }
    }

    /**
     * Verifies a [PublishedJournalistToUserDeadDrop] by checking that its certificate using the
     * [coverNodeKeyHierarchies]. Any of the [coverNodeKeyHierarchies] candidates can be used to
     * verify the signature.
     */
    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun verifyAndParseDeadDrop(
        candidate: PublishedJournalistToUserDeadDrop,
        coverNodeKeyHierarchies: List<VerifiedCoverNodeKeyHierarchy>,
    ): VerifiedDeadDrop? {
        val candidateData = candidate.data.base64Decode()
        val candidateCertificateData = DeadDropCertificateData.from(data = candidateData)
        val candidateSignatureData = DeadDropSignatureData.from(
            libSodium = libSodium,
            data = candidateData,
            createdAt = candidate.createdAt
        )

        val signingKeys = coverNodeKeyHierarchies.allCoverNodeSigningKeys()

        for (signingKey in signingKeys) {
            try {
                verifySignatureOrCertOrThrow(
                    signingKey = signingKey,
                    candidateCertificateData = candidateCertificateData,
                    candidateCertBytes = candidate.cert.hexDecode(),
                    candidateSignatureData = candidateSignatureData,
                    candidateSignatureBytes = candidate.signature?.hexDecode()
                )

                // if the signature verifications does not throw, we found a good signing key and
                // can parse the data
                return VerifiedDeadDrop(
                    id = candidate.id,
                    createdAt = candidate.createdAt,
                    messages = parseDeadDropData(candidateData)
                )
            } catch (e: Exception) {
                continue
            }
        }

        // No signing key matched; based on the requested behaviour, we either throw an exception
        // or ignore the dead-drop
        return verificationFailureBehaviour.onFailure()
    }

    /**
     * Verifies the signature of the [PublishedJournalistToUserDeadDrop] using the [signingKey].
     *
     * During the migration time, we only check the `signature` field if it has a meaningful value.
     * Otherwise, we fallback to the "normal" check against the `cert` field. This fallback
     * behaviour is only temporary and should be removed once the migration is complete, see #2998.
     */
    private fun verifySignatureOrCertOrThrow(
        signingKey: VerifiedSignedSigningKey,
        candidateCertificateData: DeadDropCertificateData,
        candidateCertBytes: ByteArray,
        candidateSignatureData: DeadDropSignatureData,
        candidateSignatureBytes: ByteArray?,
    ) {
        val meaningfulSignature = candidateSignatureBytes != null &&
                candidateSignatureBytes.any { it != 0x00.toByte() }

        if (meaningfulSignature) {
            Signature.verifyOrThrow(
                libSodium = libSodium,
                signingPk = signingKey.pk,
                data = candidateSignatureData,
                signature = Signature(candidateSignatureBytes!!),
            )
        } else {
            // if the signature is not meaningful, we fallback to the cert check; see #2998
            Signature.verifyOrThrow(
                libSodium = libSodium,
                signingPk = signingKey.pk,
                data = candidateCertificateData,
                signature = Signature(candidateCertBytes),
            )
        }
    }

    /**
     * Splits a hex-encoded [data] String into a list of [TwoPartyBox] messages.
     */
    @VisibleForTesting
    internal fun parseDeadDropData(data: ByteArray): List<TwoPartyBox<EncryptableVector>> {
        require(data.size % JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN == 0)

        return data
            .asSequence()
            .chunked(JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN) {
                TwoPartyBox<EncryptableVector>(it.toByteArray())
            }
            .toList()
    }
}
