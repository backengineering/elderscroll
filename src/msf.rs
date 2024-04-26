// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved

use crate::{
    directory::StreamDirectory, pagelist::PageList, struct_overlay_both, view::SourceView,
};
use scroll::{Error, Pread};
use static_assertions::const_assert;

/// Magic bytes of the PDB file format 7.0
pub const MAGIC: &[u8] = b"Microsoft C/C++ MSF 7.00\r\n\x1a\x44\x53\x00\x00\x00";
pub type PageNumber = u32;

// https://llvm.org/docs/PDB/MsfFile.html
// struct SuperBlock {
//     char FileMagic[sizeof(Magic)];
//     ulittle32_t BlockSize;
//     ulittle32_t FreeBlockMapBlock;
//     ulittle32_t NumBlocks;
//     ulittle32_t NumDirectoryBytes;
//     ulittle32_t Unknown;
//     ulittle32_t BlockMapAddr;
// };
struct_overlay_both!((pub MsfBigHeader, pub MsfBigHeaderMut) {
    // Must be equal to "Microsoft C / C++ MSF 7.00\\r\\n" followed by the bytes 1A 44 53 00 00 00.
    [0x00] magic: [u8; 32],
    // The block size of the internal file system. Valid values are 512, 1024, 2048, and 4096 bytes.
    // Certain aspects of the MSF file layout vary depending on the block sizes. For the purposes of LLVM,
    // we handle only block sizes of 4KiB, and all further discussion assumes a block size of 4KiB.
    [0x20] page_size: u32,
    // The index of a block within the file, at which begins a bitfield representing the set of all blocks within the file which are “free”
    // (i.e. the data within that block is not used). See The Free Block Map for more information. Important: FreeBlockMapBlock can only be 1 or 2!
    [0x24] free_page_map: u32,
    // The total number of blocks in the file. NumBlocks * BlockSize should equal the size of the file on disk.
    [0x28] num_pages: u32,
    // The size of the stream directory, in bytes. The stream directory contains information about each stream’s
    // size and the set of blocks that it occupies. It will be described in more detail later.
    [0x2C] stream_dir_size: u32,
    [0x30] unknown: u32,
    // The index of a block within the MSF file. At this block is an array of ulittle32_t’s listing the blocks that the stream directory
    // resides on. For large MSF files, the stream directory (which describes the block layout of each stream) may not fit entirely on
    // a single block. As a result, this extra layer of indirection is introduced, whereby this block contains the list of blocks that the
    // stream directory occupies, and the stream directory itself can be stitched together accordingly. The number of ulittle32_t’s in
    // this array is given by ceil(NumDirectoryBytes / BlockSize).
    [0x34] stream_block_map: u32,
});

const_assert!(MsfBigHeader::size() == 0x38);

impl<'a> MsfBigHeaderMut<'a> {
    /// How many pages are required to store N amount of bytes?
    #[inline(always)]
    pub fn pages_needed_to_store(&self, bytes: u32) -> u32 {
        (bytes + (self.get_page_size() - 1)) / self.get_page_size()
    }
    /// Get the page at which the stream block map exists.
    #[inline(always)]
    pub fn stream_block_map(&self) -> usize {
        (self.get_stream_block_map() * self.get_page_size()) as usize
    }
    /// Flush header to the buffer.
    #[inline(always)]
    pub fn flush(&self, buff: &mut Vec<u8>) {
        buff[0..Self::size()].copy_from_slice(&self.ptr);
    }
}

impl<'a> MsfBigHeader<'a> {
    /// Validates the magic bytes in the header.
    pub fn from(bytes: &'a [u8]) -> Option<Self> {
        let header = Self::new(bytes)?;
        if header.get_magic() == MAGIC {
            Some(header)
        } else {
            None
        }
    }
    /// How many pages are required to store N amount of bytes?
    #[inline(always)]
    pub fn pages_needed_to_store(&self, bytes: u32) -> u32 {
        (bytes + (self.get_page_size() - 1)) / self.get_page_size()
    }
    /// Get the page at which the stream block map exists.
    #[inline(always)]
    pub fn stream_block_map(&self) -> usize {
        (self.get_stream_block_map() * self.get_page_size()) as usize
    }
}

/// High level abstraction of a PDB/MSF file. Create one of these to manipulate the PDB/MSF.
#[derive(Debug, Default)]
pub struct BigMsf {
    /// Internal back buffer of the MSF file.
    pub bytes: Vec<u8>,
}

impl BigMsf {
    /// Create a new MSF/PDB given a copy of its bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
    /// Get an immutable reference to the MSF header.
    #[inline(always)]
    pub fn header(&self) -> Option<MsfBigHeader<'_>> {
        MsfBigHeader::new(&self.bytes)
    }
    /// Get a mutable reference to the MSF header.
    #[inline(always)]
    pub fn header_mut(&mut self) -> Option<MsfBigHeaderMut<'_>> {
        MsfBigHeaderMut::new(&mut self.bytes)
    }
    /// Find and parse the stream directory, return a read only mapping of it.
    pub fn get_stream_directory(&self) -> Result<StreamDirectory, Error> {
        let header = self
            .header()
            .ok_or_else(|| Error::Custom(format!("Failed to parse MSF header!")))?;
        // Get the page that contains page numbers for each page that the
        // stream directory uses. (Yes the stream directory might need multiple pages.)
        let stream_block_map = &self.bytes[header.stream_block_map()..];
        let num_pages = header.pages_needed_to_store(header.get_stream_dir_size());
        let mut offset = 0;
        let mut pages = PageList::new(header.get_page_size());
        // Now read all of the page numbers needed into a PageList.
        for _ in 0..num_pages {
            pages.push(stream_block_map.gread::<u32>(&mut offset)?);
        }
        // Map the pages to a linear sequence of bytes with a known size.
        let view = SourceView::with_size(&self.bytes, pages, header.get_stream_dir_size() as usize)
            .ok_or_else(|| Error::Custom("Failed to parse stream directory!".to_string()))?;
        // Parse the stream directory and return it.
        StreamDirectory::new(&self.bytes, view, &header)
    }
    /// Flush stream directory back to underlying bytes. Updates the MSF
    /// header as well.
    pub fn set_stream_directory(&mut self, mut dir: StreamDirectory) -> Result<(), Error> {
        // Make a clone of the headers right now.
        let mut header_bytes = vec![0u8; MsfBigHeaderMut::size()];
        header_bytes.copy_from_slice(
            self.header()
                .ok_or_else(|| Error::Custom(format!("Failed to parse MSF header!")))?
                .ptr,
        );
        // Cloned mutable header which we gets updated by flush.
        let mut header = MsfBigHeaderMut::new(&mut header_bytes)
            .ok_or_else(|| Error::Custom(format!("Failed to parse MSF header!")))?;
        // Flush directory back to the underlying buffer.
        dir.flush(&mut self.bytes, &mut header)?;
        // Flush updated MSF header now.
        header.flush(&mut self.bytes);
        Ok(())
    }
}
