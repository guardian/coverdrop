package com.theguardian.coverdrop.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.Card
import androidx.compose.material.Divider
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.theguardian.coverdrop.core.ui.models.UiPassphraseWord
import com.theguardian.coverdrop.ui.R
import com.theguardian.coverdrop.ui.theme.CoverDropPreviewSurface
import com.theguardian.coverdrop.ui.theme.NeutralMiddle
import com.theguardian.coverdrop.ui.theme.Padding
import com.theguardian.coverdrop.ui.theme.RoundedCornerShape
import com.theguardian.coverdrop.ui.theme.SurfaceBorder
import com.theguardian.coverdrop.ui.theme.TextWhiteMuted
import com.theguardian.coverdrop.ui.utils.parseHighlightedTextIntoAnnotated
import com.theguardian.coverdrop.ui.utils.rememberScreenInsets

internal enum class HeadlineLevel {
    H1, H2, H3
}

internal sealed class HelpScreenComponent {
    data class Headline(val text: String, val headlineLevel: HeadlineLevel) :
        HelpScreenComponent() {
        init {
            require(text.contains("\n").not()) {
                "Headline text must be exactly one line"
            }
        }
    }

    data class Text(val text: String) : HelpScreenComponent()

    data class ListItem(val text: String) : HelpScreenComponent()

    data object Divider : HelpScreenComponent()

    data object Space : HelpScreenComponent()

    data class Example(val annotatedText: AnnotatedString) : HelpScreenComponent()

    data class BlockQuote(
        val text: String,
        val authorName: String,
        val authorTagLine: String,
    ) : HelpScreenComponent()

    data class Button(
        val firstLine: String,
        val secondLine: String,
        val identifier: String,
    ) : HelpScreenComponent()

    data class PassphraseBoxes(val words: List<String>) : HelpScreenComponent()
}

/**
 * We use a simple markup language to define the help screen content. The markup language is
 * inspired by markdown, but is much simpler. In a first step the entire string is divided into
 * paragraphs (by looking for two consecutive new lines `\n\n`) The following elements are
 * supported:
 *
 * - Headlines: Lines starting with #, ##, or ### are interpreted as headlines. The number of
 *  hashes determines the headline level. Must be exactly on line.
 *
 * - Blockquotes: Lines starting with BLOCKQUOTE are interpreted as blockquotes. The blockquote
 * text is expected to be formatted as follows:
 * ```
 * BLOCKQUOTE
 * Quote text
 * Author name
 * Author tagline
 * ```
 *
 * - Examples: Lines starting with EXAMPLE are interpreted as examples. The example text is
 * expected to be formatted as follows:
 * ```
 * EXAMPLE
 * This is an example text with ~a highlighted section~ in it.
 * ```
 *
 * - Buttons: Lines starting with BUTTON are interpreted as buttons. The button identifier is
 * specified and located by the integrating screen. The button text is expected
 * to be formatted as follows:
 * ```
 * BUTTON
 * First line
 * Second line
 * Button identifier
 * ```
 *
 * - PassphraseBoxes: Lines starting with PASSPHRASE_BOXES are interpreted as passphrase boxes.
 * The passphrase words are expected to be formatted as follows:
 * ```
 * PASSPHRASE_BOXES apple waterfall diamond
 * ```
 *
 * - Dividers: Lines containing only the word DIVIDER are interpreted as dividers.
 *
 * - Space: Lines containing only the word SPACE are interpreted as space.
 *
 * - List items: Lines starting with a dash are interpreted as list items.
 *
 * - Text: Any line that is not a headline, blockquote, example, or button is interpreted as
 *  regular text.
 */
internal fun parseHelpScreenMarkup(
    markup: String,
    highlightColor: Color
): List<HelpScreenComponent> {
    val sections = markup.split("\n\n")
    val components = sections.map { section ->
        when {
            section.startsWith("# ") -> {
                val headlineText = section.removePrefix("# ")
                HelpScreenComponent.Headline(headlineText, HeadlineLevel.H1)
            }

            section.startsWith("## ") -> {
                val headlineText = section.removePrefix("## ")
                HelpScreenComponent.Headline(headlineText, HeadlineLevel.H2)
            }

            section.startsWith("### ") -> {
                val headlineText = section.removePrefix("### ")
                HelpScreenComponent.Headline(headlineText, HeadlineLevel.H3)
            }

            section.startsWith("- ") -> {
                val listItemText = section.removePrefix("- ")
                HelpScreenComponent.ListItem(listItemText)
            }

            section.startsWith("DIVIDER") -> HelpScreenComponent.Divider

            section.startsWith("SPACE") -> HelpScreenComponent.Space

            section.startsWith("EXAMPLE") -> {
                val exampleText = section.removePrefix("EXAMPLE\n")
                val annotatedText = parseHighlightedTextIntoAnnotated(exampleText, highlightColor)
                HelpScreenComponent.Example(annotatedText)
            }

            section.startsWith("BLOCKQUOTE") -> {
                val blockQuoteText = section.removePrefix("BLOCKQUOTE\n")
                val quoteParts = blockQuoteText.split("\n")
                HelpScreenComponent.BlockQuote(
                    text = quoteParts[0],
                    authorName = quoteParts[1],
                    authorTagLine = quoteParts[2]
                )
            }

            section.startsWith("BUTTON") -> {
                val buttonSection = section.removePrefix("BUTTON\n")
                val buttonLines = buttonSection.split("\n")
                HelpScreenComponent.Button(
                    firstLine = buttonLines[0],
                    secondLine = buttonLines[1],
                    identifier = buttonLines[2].trim()
                )
            }

            section.startsWith("PASSPHRASE_BOXES") -> {
                val words = section.removePrefix("PASSPHRASE_BOXES ").split(" ")
                HelpScreenComponent.PassphraseBoxes(words)
            }

            else -> {
                HelpScreenComponent.Text(section)
            }
        }
    }
    return components
}

@Composable
internal fun HelpScreenContent(
    components: List<HelpScreenComponent>,
    modifier: Modifier = Modifier,
    onClickMapping: Map<String, () -> Unit> = emptyMap(),
) {
    Column(
        modifier
            .verticalScroll(rememberScrollState())
            .padding(bottom = rememberScreenInsets().bottom)
    ) {
        Column(modifier = modifier.padding(16.dp)) {
            ComponentsColumn(components, onClickMapping)
        }
    }
}

@Composable
private fun ComponentsColumn(
    components: List<HelpScreenComponent>,
    onClickMapping: Map<String, () -> Unit>,
) {
    components.forEach { component ->
        when (component) {
            is HelpScreenComponent.Headline -> when (component.headlineLevel) {
                HeadlineLevel.H1 -> Text(
                    text = component.text,
                    style = MaterialTheme.typography.h1,
                    modifier = Modifier.padding(bottom = Padding.L)
                )

                HeadlineLevel.H2 -> Text(
                    text = component.text,
                    style = MaterialTheme.typography.h2,
                    modifier = Modifier.padding(bottom = Padding.L)
                )

                HeadlineLevel.H3 -> Text(
                    text = component.text,
                    style = MaterialTheme.typography.h3,
                    modifier = Modifier.padding(bottom = Padding.M)
                )
            }

            is HelpScreenComponent.Text -> Text(
                text = component.text,
                style = MaterialTheme.typography.body1.copy(color = TextWhiteMuted),
                modifier = Modifier.padding(bottom = Padding.M)
            )

            // Split into lines and add a bullet point for each
            is HelpScreenComponent.ListItem -> {
                val lines = component.text.trim().split("\n")
                lines.forEach { line ->
                    Row(modifier = Modifier.padding(bottom = Padding.S)) {
                        Text(
                            text = "• ",
                            style = MaterialTheme.typography.body1.copy(color = MaterialTheme.colors.primary),
                            modifier = Modifier.padding(start = Padding.S),
                        )
                        Text(
                            text = line.removePrefix("- "),
                            style = MaterialTheme.typography.body1.copy(color = TextWhiteMuted),
                            modifier = Modifier.padding(start = Padding.S),
                        )
                    }
                }
            }

            is HelpScreenComponent.Divider -> Divider(
                color = NeutralMiddle,
                thickness = 1.dp,
                modifier = Modifier.padding(vertical = Padding.L)
            )

            is HelpScreenComponent.Space -> Spacer(modifier = Modifier.height(Padding.L))

            is HelpScreenComponent.Example -> ExampleBox(
                text = component.annotatedText,
                modifier = Modifier.padding(bottom = Padding.L)
            )

            is HelpScreenComponent.BlockQuote -> BlockQuote(
                text = component.text,
                authorName = component.authorName,
                authorTagLine = component.authorTagLine,
                modifier = Modifier.padding(bottom = Padding.M),
            )

            is HelpScreenComponent.Button -> {
                require(onClickMapping.containsKey(component.identifier))
                TwoLineButton(component.firstLine, component.secondLine) {
                    onClickMapping[component.identifier]?.let { it() }
                }
            }

            is HelpScreenComponent.PassphraseBoxes -> PassphraseBoxes(
                component.words,
                modifier = Modifier.padding(top = Padding.L, bottom = Padding.XL)
            )
        }
    }
}

@Composable
private fun PassphraseBoxes(words: List<String>, modifier: Modifier = Modifier) {
    Column(modifier = modifier) {
        Text(
            text = stringResource(R.string.caption_passphrase_example),
            style = MaterialTheme.typography.body1.copy(fontWeight = FontWeight.W500),
            modifier = Modifier
                .align(Alignment.CenterHorizontally)
                .padding(bottom = Padding.M)
        )
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .wrapContentSize(),
            border = BorderStroke(width = 1.dp, color = SurfaceBorder),
            shape = RoundedCornerShape.S,
            backgroundColor = MaterialTheme.colors.surface,
        ) {
            Column(
                modifier = Modifier.padding(
                    bottom = Padding.L,
                    start = Padding.L,
                    end = Padding.L
                )
            ) {
                TextPassphraseColumn(
                    words.map { UiPassphraseWord(it, revealed = true) },
                    showHideRevealButton = false
                )
            }
        }
    }
}

@Preview(heightDp = 1000, showSystemUi = false)
@Composable
private fun HelpScreenContentWithAllComponents() {
    val markupString = """
        # Here goes the main headline
        
        BLOCKQUOTE
        Followed by an inspirational quote.
        Someone
        with authority
        
        DIVIDER
        
        ## This medium headline gives structure
        
        ### Small header
        
        Followed by a long text that will wrap into multiple lines, so we can test those typographic features.
        
        EXAMPLE
        We strive to ~highlight~ only the important parts. Especially, in ~long examples~.
        
        ### Example of a special component

        We have the following cool component to illustrate the passphrase example.

        PASSPHRASE_BOXES apple waterfall diamond

        ### Another small header
        
        Some more text followed by a list and a divider.

        - Lists are fun
        - Who would not agree? Especially with items that go over multiple lines.
        
        DIVIDER
        
        BUTTON
        Click here
        Button description
        button_id_somewhere
    """.trimIndent()


    CoverDropPreviewSurface {
        HelpScreenContent(
            components = parseHelpScreenMarkup(
                markup = markupString,
                highlightColor = MaterialTheme.colors.primary
            ),
            onClickMapping = mapOf(Pair("button_id_somewhere") { /* empty */ })
        )
    }
}
