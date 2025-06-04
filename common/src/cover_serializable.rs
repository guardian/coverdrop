use std::io;

/// A fixed-sized type that can be serialized with real data or as a cover version.
pub trait CoverSerializable: Sized {
    fn read<R>(reader: &mut R) -> anyhow::Result<Option<Self>>
    where
        R: io::Read + io::Seek;

    const SERIALIZED_LEN: usize;

    fn unchecked_write_real(&self, writer: &mut impl io::Write) -> anyhow::Result<()>;
    fn unchecked_write_cover(writer: &mut impl io::Write) -> anyhow::Result<()>;

    fn write_real<W>(&self, writer: &mut W) -> anyhow::Result<()>
    where
        W: io::Write + io::Seek,
    {
        let start = writer.stream_position()?;
        let result = self.unchecked_write_real(writer);
        let end = writer.stream_position()?;

        assert_eq!(end - start, Self::SERIALIZED_LEN as u64);

        result
    }

    fn write_cover<W>(writer: &mut W) -> anyhow::Result<()>
    where
        W: io::Write + io::Seek,
    {
        let start = writer.stream_position()?;
        let result = Self::unchecked_write_cover(writer);
        let end = writer.stream_position()?;

        assert_eq!(end - start, Self::SERIALIZED_LEN as u64);

        result
    }
}
