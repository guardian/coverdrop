use std::{fmt::Display, path::PathBuf, str::FromStr};

use itertools::Itertools;

pub enum ListedFile {
    Short {
        filename: PathBuf,
    },
    Long {
        file_mode: String,
        // `ls -l` includes the number of hard links which we will ignore here
        owner_name: String,
        group_name: String,
        bytes: usize,
        // `ls -l` includes the last modified time which we will ignore here
        filename: PathBuf,
    },
}

impl ListedFile {
    pub fn from_ls_line(ls_line: &str) -> anyhow::Result<Self> {
        let filename = PathBuf::from_str(ls_line)?;
        Ok(ListedFile::Short { filename })
    }

    pub fn from_ls_long_line(ls_line: &str) -> anyhow::Result<Self> {
        let mut inputs = ls_line.split_whitespace();

        let Some(file_mode) = inputs.next() else {
            anyhow::bail!("Could not get file mode from input line: {}", ls_line);
        };

        // Skip over hard links.
        _ = inputs.next();

        let Some(owner_name) = inputs.next() else {
            anyhow::bail!("Could not get owner name from input line: {}", ls_line);
        };

        let Some(group_name) = inputs.next() else {
            anyhow::bail!("Could not get group name from input line: {}", ls_line);
        };

        let Some(bytes) = inputs.next() else {
            anyhow::bail!("Could not get bytes from input line: {}", ls_line);
        };

        // Ignore
        let _month = inputs.next();
        let _day = inputs.next();
        let _time_or_year = inputs.next();

        let filename = Itertools::intersperse(inputs, " ").collect::<String>();
        let filename = PathBuf::from_str(&filename)?;

        Ok(ListedFile::Long {
            file_mode: file_mode.to_string(),
            owner_name: owner_name.to_string(),
            group_name: group_name.to_string(),
            bytes: bytes.parse::<usize>()?,
            filename,
        })
    }
}

impl Display for ListedFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListedFile::Short { filename } => {
                write!(f, "{}", filename.display())
            }
            ListedFile::Long {
                file_mode,
                owner_name,
                group_name,
                bytes,
                filename,
            } => {
                write!(
                    f,
                    "{} {} {} {} {}",
                    file_mode,
                    owner_name,
                    group_name,
                    bytes,
                    filename.display()
                )
            }
        }
    }
}
