// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use crate::struct_overlay_both;
use static_assertions::const_assert;

// https://llvm.org/docs/PDB/DbiStream.html#stream-header
struct_overlay_both!((pub DbiStreamHeader, pub DbiStreamHeaderMut) {
    [0x00] version: i32,
    [0x04] version_header: u32,
    [0x08] age: u32,
    [0x0C] global_stream_index: u16,
    [0x0E] build_number: u16,
    [0x10] public_stream_index: u16,
    [0x12] pdb_dll_version: u16,
    [0x14] sym_record_stream: u16,
    [0x16] pdb_dll_rbld: u16,
    [0x18] mod_info_size: u32,
    [0x1C] section_contribution_size: u32,
    [0x20] section_map_size: u32,
    [0x24] source_info_size: u32,
    [0x28] type_server_map_size: u32,
    [0x2C] mfc_type_server_index: u32,
    [0x30] optional_dbg_header_size: u32,
    [0x34] ec_substream_size: u32,
    [0x38] flags: u16,
    [0x3A] machine: u16,
    [0x3C] padding: u32,
});
const_assert!(DbiStreamHeader::size() == 0x40);

// https://llvm.org/docs/PDB/DbiStream.html#optional-debug-header-stream
struct_overlay_both!((pub DbiExtraStream, pub DbiExtraStreamMut) {
    [0x00] fpo_data: u16,
    [0x02] exception_data: u16,
    [0x04] fixup_data: u16,
    [0x06] omap_to_src: u16,
    [0x08] omap_from_src: u16,
    [0x0A] section_headers: u16,
    [0x0C] unknown1: u16,
    [0x0E] xdata: u16,
    [0x10] pdata: u16,
    [0x12] fpo2_data: u16,
    [0x14] original_section_headers: u16,
});
const_assert!(DbiExtraStream::size() == 0x16);