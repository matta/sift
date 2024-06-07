#![allow(dead_code)]
use std::io::{Error, Read, Write};

fn read_u32<R: Read>(reader: &mut R) -> Result<u32, Error> {
    let mut bytes = [0; 4];
    reader.read_exact(&mut bytes)?;
    let value = u32::from_be_bytes(bytes);
    Ok(value)
}

fn write_u32<W: Write>(writer: &mut W, value: u32) -> Result<(), Error> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

// Define a struct for the chunk
#[derive(Debug)]
pub(crate) struct Chunk {
    pub(crate) chunk_type: [u8; 4],
    pub(crate) data: Vec<u8>,
}

// Implement methods for reading and writing PngChunk from streams
impl Chunk {
    pub(crate) fn new(chunk_type: [u8; 4], data: Vec<u8>) -> Self {
        Self { chunk_type, data }
    }

    fn read<R: Read>(reader: &mut R) -> Result<Chunk, std::io::Error> {
        let data_length = read_u32(reader)?;
        let chunk_size_limit = 1024 * 1024 * 1024;
        if data_length > chunk_size_limit {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "chunk size exceeded",
            ));
        }
        let mut chunk_type = [0; 4];
        reader.read_exact(&mut chunk_type)?;
        let mut data = vec![0; usize::try_from(data_length).unwrap()];
        reader.read_exact(&mut data)?;

        let chunk = Chunk { chunk_type, data };

        let crc = read_u32(reader)?;
        if crc != chunk.compute_crc() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "CRC mismatch",
            ));
        }

        Ok(chunk)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        let Ok(len) = u32::try_from(self.data.len()) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "chunk size exceeded",
            ));
        };
        write_u32(writer, len)?;
        writer.write_all(&self.chunk_type)?;
        writer.write_all(&self.data)?;
        write_u32(writer, self.compute_crc())?;
        Ok(())
    }

    fn compute_crc(&self) -> u32 {
        let mut digest = crate::persist::crc::Digest::new();
        digest.update(&self.chunk_type);
        digest.update(self.data.as_slice());
        digest.finalize()
    }
}

// The following byte array represents the header of the file. The same value as
// a C-style string is "\x89SIFT\r\n\x1A\n". This format is taken from PNG and
// has useful properties, primarily the leading byte which is outside the ASCII
// range, including a human readable string "SIFT" string, and various line
// ending characters that will change if the file is ever incorrectly converted
// between different line endings.
const HEADER: &[u8] = &[0x89, 0x53, 0x49, 0x46, 0x54, 0x0D, 0x0A, 0x1A, 0x0A];

// Function to read the header of the file
pub(crate) fn read_header<R: Read>(
    reader: &mut R,
) -> Result<(), std::io::Error> {
    let mut header = [0; 8];
    reader.read_exact(&mut header)?;
    if header == HEADER {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid header",
        ))
    }
}

// Function to write the header of the file.
pub(crate) fn write_header<W: Write>(
    writer: &mut W,
) -> Result<(), std::io::Error> {
    writer.write_all(HEADER)
}

// Function to read a single PNG chunk
pub(crate) fn read_chunk<R: Read>(
    reader: &mut R,
) -> Result<Chunk, std::io::Error> {
    Chunk::read(reader)
}

// Function to write a single PNG chunk
pub(crate) fn write_chunk<W: Write>(
    chunk: &Chunk,
    writer: &mut W,
) -> Result<(), std::io::Error> {
    chunk.write(writer)
}
