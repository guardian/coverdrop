use itertools::Itertools;
use rand::seq::SliceRandom;
use regex::Regex;

use super::{error::GeneratorError, word_list::WordList};

/// A password generator to create and verify passwords from the EFF word list
pub struct PasswordGenerator {
    word_list: WordList,
}

impl PasswordGenerator {
    /// Create a new password generator based of the EFF large wordlist.
    pub fn from_eff_large_wordlist() -> Result<PasswordGenerator, GeneratorError> {
        Ok(PasswordGenerator {
            word_list: WordList::from_eff_large_wordlist()?,
        })
    }

    /// The total number of words in the password generator's dictionary
    pub fn words_len(&self) -> usize {
        self.word_list.words.len()
    }

    /// Create a new password with a given number of words
    pub fn generate(&self, word_count: usize) -> String {
        let mut rng = rand::thread_rng();

        let words = (0..word_count).map(|_| -> &str {
            self.word_list
                .words
                .choose(&mut rng)
                .expect("Word list to have words")
        });

        // Slightly wonky shadowing here so we can use the fully qualified path syntax
        // for `intersperse`. This is required because the standard library intends to introduce
        // `intersperse` soon.
        Itertools::intersperse(words, " ").collect()
    }

    /// Verify a password, checking it is the right format, all the words are within the dictionary.
    ///
    /// If everything is successful this function returns `Ok` with the verified password embedded inside it.
    /// It is very important that you use this password in any further functions, such as key derivation, since validated
    /// passwords are ran through `to_ascii_lowercase`.
    ///
    /// If there are any problems then we return `Err(GeneratorError)` which provides information on the
    /// nature of the problem.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::error::Error;
    ///
    /// use common::PasswordGenerator;
    /// use some_library::pbkdf;
    ///
    /// fn check_password_and_derive_key(password: &str, generator: &PasswordGenerator) -> Result<Key, Box<dyn Error>> {
    ///     // Validate the password, and get the checked password
    ///     let validated_password = generator.check_valid(&password)?;
    ///
    ///     // Pass the validated password into further functions
    ///     pbkdf(validated_password)?
    /// }
    /// ```
    pub fn check_valid(&self, password: &str) -> Result<String, GeneratorError> {
        let password = password.to_owned().to_ascii_lowercase();

        let re = Regex::new(r"^([a-zA-Z ]+)$").unwrap();

        let captures = re
            .captures(&password)
            .ok_or(GeneratorError::PasswordFormatError)?;

        let words = captures
            .get(1)
            .ok_or(GeneratorError::PasswordFormatError)?
            .as_str();

        // Find all the words which don't exist
        let invalid_words: Vec<String> = words
            .split(' ')
            .filter_map(|w| {
                let w = w.to_owned();
                if self.word_list.words.binary_search(&w.as_str()).is_ok() {
                    None
                } else {
                    Some(w)
                }
            })
            .collect();

        if invalid_words.is_empty() {
            Ok(password)
        } else {
            Err(GeneratorError::MisspeltWords(invalid_words))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        crypto::pbkdf::DEFAULT_PASSPHRASE_WORDS,
        generators::{GeneratorError, PasswordGenerator},
    };

    #[test]
    fn loads_file_successfully() -> Result<(), GeneratorError> {
        let generator = PasswordGenerator::from_eff_large_wordlist()?;
        assert_eq!(
            // The large wordlist, contains enough words for passwords to be created from 5d6
            6 * 6 * 6 * 6 * 6,
            generator.words_len(),
            "Checking the number of words loaded is correct",
        );
        Ok(())
    }

    #[test]
    fn roundtrip() -> Result<(), GeneratorError> {
        let generator = PasswordGenerator::from_eff_large_wordlist()?;
        let password = generator.generate(DEFAULT_PASSPHRASE_WORDS);
        let verify = generator.check_valid(&password);
        assert!(
            verify.is_ok(),
            "Checking if '{password}' verification is Ok(()), was {verify:?} "
        );
        Ok(())
    }

    // Just a basic test to make sure we've not broken the randomness catastrophically
    // Not a full test confirming uniform distribution etc.
    #[test]
    fn is_random() -> Result<(), GeneratorError> {
        let generator = PasswordGenerator::from_eff_large_wordlist()?;
        let password_one = generator.generate(10);
        let password_two = generator.generate(10);

        assert_ne!(password_one, password_two);

        Ok(())
    }

    // Since our word list is all lower case it's never valid to have an upper case character anywhere
    // in the password. As such, it feels pointless to punish our users if they accidentally capitalise
    // something. So we always lower case the password when checking it's valid.
    #[test]
    fn check_hardcoded_string_case_insensitive() -> Result<(), GeneratorError> {
        let generator = PasswordGenerator::from_eff_large_wordlist()?;
        let password = "external jersey SQUEEZE luckiness collector";
        let validated = generator.check_valid(password);
        assert!(
            validated.is_ok(),
            "Checking if '{password}' validation is Ok(()), was {validated:?} "
        );
        assert_eq!("external jersey squeeze luckiness collector", validated?);
        Ok(())
    }

    #[test]
    fn check_missseplt_word() -> Result<(), GeneratorError> {
        let generator = PasswordGenerator::from_eff_large_wordlist()?;
        let password = "external jersey squeeze luckyness collector";
        let validated = generator.check_valid(password);

        assert!(
            matches!(validated, Err(GeneratorError::MisspeltWords(_))),
            "Checking misspelt word causes error",
        );
        Ok(())
    }

    // This could possibly be replaced with some genuine fuzzing, but this will do for now
    #[test]
    fn format_errors_fail() -> Result<(), GeneratorError> {
        let generator = PasswordGenerator::from_eff_large_wordlist()?;
        let passwords: Vec<String> = vec![
            // Check that empty password is invalid
            "".into(),
            // Check that there's no numbers mixed in with the word section
            "abc 123 abc".into(),
            // Check accidental accents, useful for non-UK/US keyboard layouts?
            "wérd with accent".into(),
        ];

        passwords.iter().for_each(|password| {
            let validated = generator.check_valid(password);

            assert!(
                matches!(validated, Err(GeneratorError::PasswordFormatError)),
                "Checking that invalid password causes a formatting error '{password}' was {validated:?}"
            );
        });
        Ok(())
    }
}
