package com.theguardian.coverdrop.ui.utils

import com.theguardian.coverdrop.core.crypto.Passphrase
import com.theguardian.coverdrop.core.models.JournalistInfo
import com.theguardian.coverdrop.core.models.JournalistVisibility
import com.theguardian.coverdrop.core.models.Message
import com.theguardian.coverdrop.core.models.MessageThread
import com.theguardian.coverdrop.core.utils.DefaultClock
import com.theguardian.coverdrop.ui.R
import java.time.Duration
import java.time.Instant
import kotlin.random.Random

val COVERDROP_SAMPLE_DATA = SampleDataProvider()

private const val LOREM_IPSUM = """Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Integer dolor  nulla, ornare et tristique imperdiet, dictum sit amet velit. Curabitur pharetra erat
sed neque interdum, non mattis tortor auctor. Curabitur eu ipsum ac neque semper eleifend.
Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus.
Integer erat mi, ultrices nec arcu ut, sagittis sollicitudin est. In hac habitasse
platea dictumst. Sed in efficitur elit. Curabitur nec commodo elit. Aliquam tincidunt
rutrum nisl ut facilisis. Aenean ornare ut mauris eget lacinia. Mauris a felis quis orci
auctor varius sit amet eget est. Curabitur a urna sit amet diam sagittis aliquet eget eu
sapien. Curabitur a pharetra purus.
Nulla facilisi. Suspendisse potenti. Morbi mollis aliquet sapien sed faucibus. Donec
aliquam nibh nibh, ac faucibus felis aliquam at. Pellentesque egestas enim sem, eu
tempor urna posuere eget. Cras fermentum commodo neque ac gravida. Cras ut magna
consequat mauris viverra posuere eu rhoncus arcu. Donec ut efficitur tortor, aliquam
convallis nisi. Integer commodo leo quis ornare varius. Nullam non quam dolor. Donec
tincidunt et diam quis semper. Maecenas nec lacinia libero. Morbi nec venenatis mi, vel
hendrerit mi.
Nulla imperdiet mollis cursus. Suspendisse potenti. Class aptent taciti sociosqu ad
litora torquent per conubia nostra, per inceptos himenaeos. Nulla vitae diam vel ex
dictum tincidunt in vel sem. Nunc ante nisi, rutrum eu sodales eu, elementum vitae
turpis. Vivamus accumsan auctor diam, et rhoncus enim hendrerit vitae. Nunc ac justo
tortor. Maecenas nec sapien fermentum, faucibus elit eget, consequat mi. Quisque gravida
mauris arcu. Nullam vel erat ante. Aenean finibus nunc eget lacus volutpat condimentum.
Ut imperdiet ante et elit efficitur, sit amet egestas est mollis. In blandit, magna nec
auctor fermentum, arcu ipsum pulvinar odio, eget convallis ligula eros ac nisl."""

private val SHORT_SAMPLE_PASSPHRASE = listOf("album", "brunch", "cheese", "dash")
private val LONG_SAMPLE_PASSPHRASE =
    SHORT_SAMPLE_PASSPHRASE + listOf("email", "fruit", "grip", "hug")

private val TEAM_NAMES = listOf(
    "Arts",
    "Consumer", // will be JournalistVisibility.HIDDEN
    "Education",
    "Film",
    "G20",
    "International news",
    "Investigation",
    "Law",
    "Politics",
)
private val JOURNALIST_NAMES = listOf(
    "Alice",
    "Bob", // will be JournalistVisibility.HIDDEN
    "Charlie",
    "Dave",
    "Eve",
    "Fred",
    "Gilbert",
    "Hugh",
)

class SampleDataProvider {

    fun getSampleErrorMessage(isFatal: Boolean) = UiErrorMessage(
        messageResId = if (isFatal) R.string.test_error_message_fatal else R.string.test_error_message,
        isFatal = isFatal,
    )

    fun getShortPassphrase() = Passphrase(SHORT_SAMPLE_PASSPHRASE.map { it.toCharArray() })

    fun getLongPassphrase() = Passphrase(LONG_SAMPLE_PASSPHRASE.map { it.toCharArray() })

    fun getWordList() = (SHORT_SAMPLE_PASSPHRASE + LONG_SAMPLE_PASSPHRASE).toSet().toList()

    fun getSampleMessage(wordCount: Int = 42) = getSampleText(wordCount = wordCount)

    fun getSampleThread(
        numMessages: Int = 10,
        now: Instant = Instant.now(),
        seed: Int = 0,
        lastMessageIsSent: Boolean = false,
    ): MessageThread {
        val random = Random(seed = seed)
        val journalistInfo = getSampleJournalistInfo()

        val timesteps = Duration.ofHours(1)
        val startTime = now - timesteps.multipliedBy(numMessages.toLong())

        val messages = List(numMessages) { index ->
            val time = startTime + timesteps.multipliedBy(index.toLong())
            if (index == numMessages - 1 && !lastMessageIsSent) {
                Message.Pending(getSampleText(random.nextInt(20)) + " id$index", time)
            } else if (index % 2 == 0) {
                Message.Sent(getSampleText(random.nextInt(20)) + " id$index", time)
            } else {
                Message.Received(getSampleText(random.nextInt(20)) + " id$index", time)
            }
        }

        return MessageThread(journalistInfo, messages)
    }

    private fun getSampleJournalistInfo(): JournalistInfo {
        return JournalistInfo(
            id = "1",
            displayName = "Charles Darwin",
            sortName = "Darwin Charles",
            description = "Father of evolution theory and wildlife reporter",
            isTeam = false,
            tag = "CRD",
            visibility = JournalistVisibility.VISIBLE,
        )
    }

    fun getTeams(): List<JournalistInfo> {
        return TEAM_NAMES.mapIndexed { idx, name ->
            JournalistInfo(
                id = "d$idx",
                displayName = "$name team",
                description = getSampleText(50),
                isTeam = true,
                tag = "a0b1c2d3",
                visibility = if (idx != 1) JournalistVisibility.VISIBLE else JournalistVisibility.HIDDEN,
            )
        }
    }

    fun getJournalists(): List<JournalistInfo> {
        return JOURNALIST_NAMES.mapIndexed { idx, name ->
            JournalistInfo(
                id = "j$idx",
                displayName = name,
                description = getSampleText(3),
                isTeam = false,
                tag = "a0b1c2d3",
                visibility = if (idx != 1) JournalistVisibility.VISIBLE else JournalistVisibility.HIDDEN,
            )
        }
    }

    private fun getSampleText(wordCount: Int) =
        generateSequence { LOREM_IPSUM.splitToSequence(" ") }
            .flatten()
            .take(wordCount)
            .joinToString(" ")
}
