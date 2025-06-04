package com.theguardian.coverdrop.core.encryptedstorage

import android.annotation.SuppressLint
import androidx.annotation.VisibleForTesting
import com.lambdapioneer.sloth.impl.LongSlothParams
import com.theguardian.coverdrop.core.crypto.PassphraseKdfParameters

/**
 * The global security parameters for the encrypted storage. These are chosen based on the
 * availability of the Secure Element. We insert these parameters from the top (rather than
 * creating them in the storage implementations), so that we can inject special ones for testing.
 *
 * See https://github.com/guardian/coverdrop/issues/329 for details and discussion of these
 * parameter choices.
 *
 * @param wordCount Number of words in the passphrase
 */
internal sealed class EncryptedStorageConfiguration(internal val wordCount: Int) {

    /**
     * Uses the [EncryptedStorageWithSecureElement] implementation.
     *
     * @param secureElementParameters Configuration of the Sloth algorithm
     */
    internal class SecureElement(
        internal val secureElementParameters: SecureElementParameters,
        wordCount: Int = 3,
    ) : EncryptedStorageConfiguration(wordCount = wordCount)

    /**
     * Uses the [EncryptedStorageWithPassword] implementation.
     *
     * @param passphraseKdfParameters Configuration of the Argon2 KDF
     */
    internal class PasswordOnly(
        internal val passphraseKdfParameters: PassphraseKdfParameters,
        wordCount: Int = 5,
    ) : EncryptedStorageConfiguration(wordCount = wordCount)
}


/**
 * The [SecureElementParameters] set the hardness of the key derivation using the Secure Element.
 *
 * @param slothParameters Configuration value for the LongSloth algorithm hardness (see paper).
 * @param passphraseKdfParameters Configuration value for the Argon2 step that is part of the
 *                                LongSloth algorithm.
 */
internal enum class SecureElementParameters(
    internal val slothParameters: LongSlothParams,
    internal val passphraseKdfParameters: PassphraseKdfParameters
) {
    /**
     * The default parameter choice for all user facing application variants.
     */
    SECURE(
        slothParameters = LongSlothParams(l = 100_000),
        passphraseKdfParameters = PassphraseKdfParameters.INTERACTIVE
    ),

    /**
     * A reduced parameter to make tests faster on old device.
     */
    @VisibleForTesting
    TEST_INSECURE(
        slothParameters = LongSlothParams(l = 10_000),
        passphraseKdfParameters = PassphraseKdfParameters.TEST_INSECURE
    ),
}

/**
 * Creates a new [EncryptedStorageConfiguration] based on the SE availability and whether we are
 * calling this for local testing or automated tests.
 */
@SuppressLint("VisibleForTests")
internal fun selectEncryptedStorageConfiguration(
    secureElementAvailabilityCache: SecureElementAvailabilityCache,
    isForLocalTestMode: Boolean = false,
    isForAutomatedTest: Boolean = false,
): EncryptedStorageConfiguration {
    return selectEncryptedStorageConfiguration(
        useSecureElement = secureElementAvailabilityCache.isAvailable(),
        isForLocalTestMode = isForLocalTestMode,
        isForAutomatedTest = isForAutomatedTest
    )
}

/**
 * Creates a new [EncryptedStorageConfiguration] based on the SE availability and whether we are
 * calling this for local testing or automated tests.
 */
@SuppressLint("VisibleForTests")
internal fun selectEncryptedStorageConfiguration(
    useSecureElement: Boolean,
    isForLocalTestMode: Boolean = false,
    isForAutomatedTest: Boolean = false,
): EncryptedStorageConfiguration {
    require(!(isForLocalTestMode && isForAutomatedTest)) { "Cannot create configuration for both local and automated testing." }

    if (isForAutomatedTest) {
        val wordCountForAutomatedTests = 4 // ensures testing multiple entry fields
        return when (useSecureElement) {
            true -> EncryptedStorageConfiguration.SecureElement(
                secureElementParameters = SecureElementParameters.TEST_INSECURE,
                wordCount = wordCountForAutomatedTests
            )

            false -> EncryptedStorageConfiguration.PasswordOnly(
                passphraseKdfParameters = PassphraseKdfParameters.TEST_INSECURE,
                wordCount = wordCountForAutomatedTests
            )
        }
    }

    if (isForLocalTestMode) {
        val wordCountForLocalTests = 1 // allowing easier manual testing
        return when (useSecureElement) {
            true -> EncryptedStorageConfiguration.SecureElement(
                secureElementParameters = SecureElementParameters.SECURE,
                wordCount = wordCountForLocalTests
            )

            false -> EncryptedStorageConfiguration.PasswordOnly(
                passphraseKdfParameters = PassphraseKdfParameters.HIGH,
                wordCount = wordCountForLocalTests
            )
        }
    }

    return when (useSecureElement) {
        true -> EncryptedStorageConfiguration.SecureElement(
            secureElementParameters = SecureElementParameters.SECURE,
        )

        false -> EncryptedStorageConfiguration.PasswordOnly(
            passphraseKdfParameters = PassphraseKdfParameters.HIGH,
        )
    }
}
