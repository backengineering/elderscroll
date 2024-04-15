// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use std::str::FromStr;

use crate::{
    directory::StreamDirectory, pagelist::PageList, struct_overlay_both, view::SourceView,
};
use scroll::{ctx, Error, Pread};
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
    /// Find and parse the stream directory
    pub fn get_stream_directory(&self) -> Result<StreamDirectory, Error> {
        // Get the page that contains page numbers for each page that the
        // stream directory uses. (Yes the stream directory might need multiple pages.)
        let stream_block_map = &self.ptr[self.get_stream_block_map() as usize..];
        let num_pages = self.pages_needed_to_store(self.get_stream_dir_size());
        let mut offset = 0;
        let mut pages = PageList::new(self.get_page_size() as usize);
        // Now read all of the page numbers needed into a PageList.
        for _ in 0..num_pages {
            pages.push(stream_block_map.gread::<u32>(&mut offset)?);
        }
        // Map the pages to a linear sequence of bytes.
        let view = SourceView::new(&self.ptr, pages)
            .ok_or_else(|| Error::Custom("Failed to parse stream directory!".to_string()))?;

        todo!();
    }
    /// How many pages are required to store N amount of bytes?
    pub fn pages_needed_to_store(&self, bytes: u32) -> u32 {
        (bytes + (self.get_page_size() - 1)) / self.get_page_size()
    }
}

const_assert!(MsfBigHeader::size() == 0x38);

#[cfg(test)]
mod tests {
    use super::MsfBigHeader;

    #[test]
    fn pdb_test1() {
        assert!(MsfBigHeader::from(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/bins/HelloWorld.exe"
        )))
        .is_none());
        assert!(MsfBigHeader::from(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/bins/HelloWorld.pdb"
        )))
        .is_some());
    }
}
