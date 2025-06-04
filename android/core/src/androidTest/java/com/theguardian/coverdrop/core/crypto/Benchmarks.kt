package com.theguardian.coverdrop.core.crypto

import android.util.Log
import com.theguardian.coverdrop.core.createLibSodium
import org.junit.Test
import java.time.Duration

private const val BENCHMARK_TAG = "Benchmark"

class Benchmarks {
    private val libSodium = createLibSodium()

    @Test
    fun benchmarkTwoPartyBoxDecryption_success() {
        val sender = EncryptionKeyPair.new(libSodium)
        val recipient = EncryptionKeyPair.new(libSodium)
        val originalMessage = EncryptableVector.fromString("hello world")

        val box = TwoPartyBox.encrypt(
            libSodium = libSodium,
            recipientPk = recipient.publicEncryptionKey,
            senderSk = sender.secretEncryptionKey,
            data = originalMessage
        )

        runBenchmark(iterations = 1_000) {
            TwoPartyBox.decrypt(
                libSodium = libSodium,
                senderPk = sender.publicEncryptionKey,
                recipientSk = recipient.secretEncryptionKey,
                data = box,
                constructor = ::EncryptableVector
            )
        }
    }

    @Test
    fun benchmarkTwoPartyBoxDecryption_fail() {
        val sender = EncryptionKeyPair.new(libSodium)
        val recipient = EncryptionKeyPair.new(libSodium)
        val otherRecipient = EncryptionKeyPair.new(libSodium)
        val originalMessage = EncryptableVector.fromString("hello world")

        val box = TwoPartyBox.encrypt(
            libSodium = libSodium,
            recipientPk = otherRecipient.publicEncryptionKey, // encrypt to other recipient
            senderSk = sender.secretEncryptionKey,
            data = originalMessage
        )

        runBenchmark(iterations = 1_000) {
            try {
                TwoPartyBox.decrypt(
                    libSodium = libSodium,
                    senderPk = sender.publicEncryptionKey,
                    recipientSk = recipient.secretEncryptionKey,
                    data = box,
                    constructor = ::EncryptableVector
                )
            } catch (e: IllegalStateException) {
                // ignore
            }
        }
    }

    // In future we might want to consider to use an external benchmark suite:
    // https://developer.android.com/topic/performance/benchmarking/microbenchmark-write
    private fun runBenchmark(iterations: Int, operation: () -> Unit) {
        val warmUp = 10
        repeat(warmUp) {
            operation()
        }

        val timeStart = System.nanoTime()

        repeat(iterations) {
            operation()
        }

        val timeEnd = System.nanoTime()
        val difference = Duration.ofNanos(timeEnd - timeStart)

        Log.i(BENCHMARK_TAG, "iterations: $iterations")
        Log.i(BENCHMARK_TAG, "total: ${difference.toMillis()} ms")

        val individualTimeMs = difference.toMillis().toFloat() / iterations
        Log.i(BENCHMARK_TAG, "per: $individualTimeMs ms")
    }
}