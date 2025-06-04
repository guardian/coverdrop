use itertools::Itertools as _;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

use super::{word_list::WordList, GeneratorError};

pub struct NameGenerator {
    word_list: WordList,
}

impl NameGenerator {
    /// Create a new name generator based of the EFF large wordlist.
    pub fn from_eff_large_wordlist() -> Result<NameGenerator, GeneratorError> {
        Ok(NameGenerator {
            word_list: WordList::from_eff_large_wordlist()?,
        })
    }

    pub fn name_from_bytes(&self, bytes: &[u8], number_of_words: usize) -> String {
        let mut seed = [u8::MAX; 32];

        let limit = bytes.len().min(seed.len());
        seed[..limit].copy_from_slice(&bytes[..limit]);

        let mut rng = StdRng::from_seed(seed);

        let name = self
            .word_list
            .words
            .choose_multiple(&mut rng, number_of_words)
            .copied()
            .join(" ");

        name
    }
}

impl Default for NameGenerator {
    fn default() -> Self {
        Self::from_eff_large_wordlist().expect("Create name generator from large worldlist")
    }
}

#[cfg(test)]
mod tests {
    use super::NameGenerator;

    #[test]
    fn generates_name() -> anyhow::Result<()> {
        let generator = NameGenerator::from_eff_large_wordlist()?;

        // Generate a few names modifying the first of four bytes slightly each time
        // They should be random enough that they're not easily confused but they
        // don't have to be cryptographically secure

        let name_1 = generator.name_from_bytes(&[0x1, 0x2, 0x3, 0x4], 4);
        assert_eq!(name_1, "cloud hamper monstrous landmass");

        let name_2 = generator.name_from_bytes(&[0x2, 0x2, 0x3, 0x4], 4);
        assert_eq!(name_2, "refreeze void stuffed stoic");

        let name_3 = generator.name_from_bytes(&[0x3, 0x2, 0x3, 0x4], 4);
        assert_eq!(name_3, "factor reviver snowplow gurgling");

        let name_4 = generator.name_from_bytes(&[0x4, 0x2, 0x3, 0x4], 4);
        assert_eq!(name_4, "flatware showbiz dodge gave");

        Ok(())
    }
}
