// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use crate::struct_overlay_both;

/// Magic bytes of the PDB file format 7.0
pub const MAGIC: &[u8] = b"Microsoft C/C++ MSF 7.00\r\n\x1a\x44\x53\x00\x00\x00";

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
