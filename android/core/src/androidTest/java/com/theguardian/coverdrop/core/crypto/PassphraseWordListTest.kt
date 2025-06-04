package com.theguardian.coverdrop.core.crypto

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import org.junit.Test

class PassphraseWordListTest {

    /**
     * Creates a [PassphraseWordList] for the test.
     */
    private fun createPassphraseWordList(): PassphraseWordList =
        PassphraseWordList.createFromEffWordList(
            context = InstrumentationRegistry.getInstrumentation().targetContext
        )

    @Test
    fun testLoadingWordlist_whenLoaded_thenLengthMatchesRawFile() {
        val instance = createPassphraseWordList()

        assertThat(instance.getWordListSize()).isEqualTo(7776)
    }

    @Test
    fun testGeneratingPassphrase_whenProvidedNumber_thenHasGivenNumberOfWords() {
        val instance = createPassphraseWordList()

        val passphrase1 = instance.generatePassphrase(numberOfWords = 1)
        assertThat(passphrase1.getPassphraseString().concatToString()).matches("\\w+")

        val passphrase3 = instance.generatePassphrase(numberOfWords = 3)
        assertThat(passphrase3.getPassphraseString().concatToString()).matches("\\w+ \\w+ \\w+")
    }

    @Test
    fun testGeneratingPassphrase_whenGeneratingTwoPassphrasesOnSameInstance_thenDifferent() {
        val instance = createPassphraseWordList()

        val passphraseA = instance.generatePassphrase(numberOfWords = 3)
        val passphraseB = instance.generatePassphrase(numberOfWords = 3)

        assertThat(passphraseA).isNotEqualTo(passphraseB)
    }

    @Test
    fun testIsValidPassphrase_whenGivenValidInput_thenTrue() {
        val instance = createPassphraseWordList()

        val passphrase = instance.generatePassphrase(numberOfWords = 3)
        assertThat(instance.isValidPassphrase(passphrase)).isTrue()
    }

    @Test
    fun testIsValidPassphrase_whenGivenValidInputInUpperCase_thenTrue() {
        val instance = createPassphraseWordList()

        val passphrase = instance.generatePassphrase(numberOfWords = 3)

        val upperCaseWords = passphrase.getWords().map {
            it.concatToString().uppercase().toCharArray()
        }
        val passphraseUppercase = Passphrase(words = upperCaseWords)
        assertThat(instance.isValidPassphrase(passphraseUppercase)).isTrue()
    }


    @Test
    fun testIsValidPassphrase_whenGivenInvalidInput_thenFalse() {
        val instance = createPassphraseWordList()

        val badPassphrase = Passphrase(words = listOf("wrongwrong".toCharArray()))

        assertThat(instance.isValidPassphrase(badPassphrase)).isFalse()
    }

    @Test
    fun testPassphraseEquality_whenEqual_thenTrue_otherwiseFalse() {
        val passphrase1 = Passphrase(words = listOf("album".toCharArray(), "cheese".toCharArray()))
        val passphrase2 = Passphrase(words = listOf("album".toCharArray(), "cheese".toCharArray()))
        assertThat(passphrase1).isEqualTo(passphrase2)

        val passphrase3 = Passphrase(words = listOf("cheese".toCharArray(), "album".toCharArray()))
        assertThat(passphrase1).isNotEqualTo(passphrase3)
    }

    @Test
    fun testIsValidPrefix_whenEmptyOrValid_thenTrue() {
        val instance = createPassphraseWordList()

        assertThat(instance.isValidPrefix("")).isTrue()
        assertThat(instance.isValidPrefix("a")).isTrue()
        assertThat(instance.isValidPrefix("al")).isTrue()
        assertThat(instance.isValidPrefix("album")).isTrue()
    }

    @Test
    fun testIsValidPrefix_whenInvalid_thenFalse() {
        val instance = createPassphraseWordList()

        assertThat(instance.isValidPrefix("zz")).isFalse()
        assertThat(instance.isValidPrefix("zzz")).isFalse()
    }

}
