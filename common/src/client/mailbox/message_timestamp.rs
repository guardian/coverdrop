use std::{fmt::Display, io, mem::size_of};

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::time;

/// A timestamp for mailbox messages.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct MessageTimestamp(pub DateTime<Utc>);

impl MessageTimestamp {
    pub const SERIALIZED_LEN: usize = size_of::<i64>();

    pub fn new(timestamp: DateTime<Utc>) -> Self {
        Self(timestamp)
    }

    pub fn now() -> Self {
        Self(time::now())
    }

    pub fn read(reader: &mut impl io::Read) -> anyhow::Result<Self> {
        let mut bytes = [0; Self::SERIALIZED_LEN];
        reader.read_exact(&mut bytes)?;
        let timestamp = i64::from_be_bytes(bytes);

        let datetime = Utc.timestamp_nanos(timestamp);

        Ok(Self(datetime))
    }

    pub fn write(&self, writer: &mut impl io::Write) -> anyhow::Result<()> {
        writer.write_all(
            &self
                .0
                .timestamp_nanos_opt()
                .expect("Date should be between 1677-09-21~2262-04-11")
                .to_be_bytes(),
        )?;

        Ok(())
    }
}

impl Display for MessageTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
