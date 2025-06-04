use super::error::GeneratorError;

pub struct WordList {
    pub words: Vec<&'static str>,
}

impl WordList {
    fn parse_eff_large_wordlist(text: &'static str) -> Result<Vec<&'static str>, GeneratorError> {
        let mut words: Vec<&'static str> = text
            .split('\n')
            .filter(|line| !line.is_empty())
            .map(|line| {
                if let Some(word) = line.split('\t').next_back() {
                    Ok(word)
                } else {
                    Err(GeneratorError::InvalidWordListLine)
                }
            })
            .collect::<Result<Vec<&'static str>, GeneratorError>>()?;

        // Sort because later we want to do a binary search for possibly misspelt words.
        words.sort();

        Ok(words)
    }

    /// Create a new password generator based of the EFF large wordlist.
    pub fn from_eff_large_wordlist() -> Result<WordList, GeneratorError> {
        let wordlist = include_str!("../../eff_large_wordlist.txt");
        let words = Self::parse_eff_large_wordlist(wordlist)?;

        Ok(WordList { words })
    }
}
