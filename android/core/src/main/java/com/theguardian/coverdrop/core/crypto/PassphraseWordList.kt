package com.theguardian.coverdrop.core.crypto

import android.content.Context
import androidx.annotation.VisibleForTesting
import com.theguardian.coverdrop.core.R
import java.security.SecureRandom
import java.util.Arrays

/**
 * In-memory representation of the user's passphrase. It is stored as a [CharArray] to allow
 * overwriting after use.
 *
 * Note that this is an obfuscation suggested by the 7ASecurity audit. While it arguably adds an
 * extra hurdle, it but does not provide complete protection against a root-level attacker which
 * can, of course, recover secrets through other means (key logger, memory analysis, ...).
 *
 * @param words A list of words to generate the passphrase from. A local copy will be created and
 *              the input will be overwritten with '0' bytes.
 */
class Passphrase(words: List<CharArray>) {
    private val normalizedWords: List<CharArray> = normalizeWords(words)
    private val normalizedPassphraseString: CharArray = joinWordsToCharArray(normalizedWords)
    private var isCleared: Boolean = false

    fun getWords(): List<CharArray> {
        check(!isCleared)
        return normalizedWords
    }

    fun getPassphraseString(): CharArray {
        check(!isCleared)
        return normalizedPassphraseString
    }

    fun isNotEmpty() = normalizedWords.isNotEmpty()

    fun clear() {
        normalizedWords.forEach { Arrays.fill(it, '0') }
        Arrays.fill(normalizedPassphraseString, '0')
        isCleared = true
    }

    /**
     * Safe-guard in case [clear] is not being called. See the Kotlin documentation that no
     * `override` keyword is needed: https://kotlinlang.org/docs/java-interop.html#finalize
     */
    protected fun finalize() {
        clear()
    }

    /**
     * Normalizes the input words by turning each character to their lowercase variant. The input
     * is "consumed" and set to `0` bytes.
     */
    private fun normalizeWords(inputWords: List<CharArray>): List<CharArray> {
        val outputWords = List(inputWords.size) { idx ->
            val inputWord = inputWords[idx]
            CharArray(inputWord.size) { inputWord[it].lowercaseChar() }
        }
        inputWords.forEach { Arrays.fill(it, '0') }
        return outputWords
    }

    /**
     * Creates a CharArray of the concatenation of all words separated by a ' ' (space) character.
     */
    private fun joinWordsToCharArray(words: List<CharArray>): CharArray {
        val l = words.sumOf { it.size } + words.size - 1
        val arr = CharArray(l) { ' ' }

        var i = 0
        for (word in words) {
            System.arraycopy(word, 0, arr, i, word.size)
            i += word.size + 1 // extra +1 for the space character between words
        }
        return arr
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        return normalizedPassphraseString.contentEquals((other as Passphrase).normalizedPassphraseString)
    }

    override fun hashCode(): Int {
        return normalizedPassphraseString.hashCode()
    }
}


/** Generator and validator for memorable passphrases using the words */
class PassphraseWordList(wordList: List<String>) {
    private val mWordsList: List<String> = wordList
    private val mWordsSet = lazy { computeWordsSet(mWordsList) }

    // This typically allocates around 2 MiB which is good enough for us. A potential optimization
    // is to replace this with a Trie. However, simple wins.
    private val mWordsPrefixSet = lazy { computeAllPrefixes(mWordsList) }

    private fun computeWordsSet(words: List<String>): Set<String> {
        return HashSet(words)
    }

    private fun computeAllPrefixes(words: List<String>): Set<String> {
        val maximumPrefixCount = words.sumOf { it.length }
        val allPrefixes = HashSet<String>(maximumPrefixCount)
        for (word in words) {
            for (len in 1..word.length) {
                allPrefixes.add(word.substring(0, len))
            }
        }
        return allPrefixes
    }

    /**
     * Generates a new passphrase using [numberOfWords] words separated by spaces. The method
     * normalizes the passphrase to lower-case.
     */
    fun generatePassphrase(numberOfWords: Int): Passphrase {
        require(numberOfWords >= 1)

        val secureRandom = SecureRandom()
        val words = List(numberOfWords) {
            mWordsList[secureRandom.nextInt(mWordsList.size)].toCharArray()
        }

        return Passphrase(words)
    }

    /**
     * Returns `true` if the provided passphrase could have been generated [generatePassphrase].
     */
    fun isValidPassphrase(passphrase: Passphrase): Boolean {
        require(passphrase.isNotEmpty())

        val wordSet = mWordsSet.value
        return passphrase.getWords().all { word -> wordSet.contains(word.concatToString()) }
    }

    /**
     * Returns `true` iff the given [string] is the prefix of any word in the word list or empty.
     */
    fun isValidPrefix(string: String): Boolean {
        if (string.isEmpty()) return true
        return mWordsPrefixSet.value.contains(string)
    }

    /**
     * Optional method to prepare the prefix checks for subsequent calls to [isValidPrefix]. This
     * is helpful to avoid initial UI lag for the very first usage.
     */
    fun preparePrefixes() {
        mWordsPrefixSet.value
    }

    @VisibleForTesting
    internal fun getWordListSize(): Int = mWordsList.size

    companion object {
        /**
         * Create a from the EFF Large Wordlist. This  performs I/O operations to load the word list.
         */
        fun createFromEffWordList(context: Context): PassphraseWordList {
            return PassphraseWordList(loadWordsList(context))
        }

        /**
         * The EFF word list contains 7776 lines each starting with a numerical representation
         * of dice configuration followed (separated by \t) by the word itself.
         */
        private fun loadWordsList(context: Context): List<String> {
            val inputStream = context.resources.openRawResource(R.raw.eff_large_wordlist)
            return inputStream.use {
                it.bufferedReader()
                    .lineSequence()
                    .map { line -> line.split("\t")[1] }
                    .toList()
            }
        }
    }
}
