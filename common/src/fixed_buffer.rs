use std::{io, mem::size_of};

use crate::cover_serializable::CoverSerializable;

/// Fixed size buffer, any entries added beyond the maximum capacity will overwrite previous entries.
/// This is useful when you want to statically confirm there can never be more than `N` values.
///
/// This is a lot like a ring buffer except we don't provide a way to `pop` entries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedBuffer<T: CoverSerializable, const N: usize> {
    next_value_index: usize,
    values: [Option<T>; N],
}

impl<T: CoverSerializable, const N: usize> FixedBuffer<T, N> {
    pub const SERIALIZED_LEN: usize = T::SERIALIZED_LEN * N + size_of::<usize>();

    const EMPTY_ENTRY: Option<T> = None;

    pub fn new() -> Self {
        Self {
            next_value_index: 0,
            values: [Self::EMPTY_ENTRY; N],
        }
    }

    pub fn from_parts(current_index: usize, values: [Option<T>; N]) -> Self {
        Self {
            next_value_index: current_index,
            values,
        }
    }

    /// Add an item to the ring buffer, replacing the oldest item in the buffer if
    /// the total number of items added is greater than the buffers capacity.
    pub fn push(&mut self, value: T) {
        let index = self.next_value_index % N;
        self.values[index] = Some(value);
        self.next_value_index += 1;

        // At some point we might want to return the evicted value here?
    }

    /// Count the number of values currently present in the ring buffer.
    pub fn count(&self) -> usize {
        self.values.iter().flatten().count()
    }

    /// The current index represents the number of items that have been pushed into the ring buffer.
    pub fn current_index(&self) -> usize {
        self.next_value_index
    }

    /// Get the underlying fixed-size array. This is useful if we want to serialize.
    pub fn underlying_array(&self) -> &[Option<T>] {
        &self.values
    }

    pub fn iter(&self) -> FixedBufferIter<T> {
        FixedBufferIter {
            values: &self.values,
            index: 0,
        }
    }

    pub fn read<R>(reader: &mut R) -> anyhow::Result<Self>
    where
        R: io::Read + io::Seek,
    {
        let mut index_buf = [0; size_of::<usize>()];
        reader.read_exact(&mut index_buf)?;
        let current_index = usize::from_be_bytes(index_buf);

        let mut values = [Self::EMPTY_ENTRY; N];

        for item in values.iter_mut() {
            *item = T::read(reader)?;
        }

        Ok(Self::from_parts(current_index, values))
    }

    pub fn write<W>(&self, writer: &mut W) -> anyhow::Result<()>
    where
        W: io::Write + io::Seek,
    {
        writer.write_all(&self.current_index().to_be_bytes())?;
        for value in self.values.iter() {
            match value {
                Some(value) => value.write_real(writer)?,
                None => T::write_cover(writer)?,
            }
        }
        Ok(())
    }
}

impl<T: CoverSerializable, const N: usize> Default for FixedBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FixedBufferIter<'a, T> {
    values: &'a [Option<T>],
    index: usize,
}

impl<'a, T> Iterator for FixedBufferIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.values.len() {
            return None;
        }

        if let Some(v) = &self.values[self.index] {
            self.index += 1;
            Some(v)
        } else {
            None
        }
    }
}
