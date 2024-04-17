// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use crate::{directory::Stream, struct_overlay_both};
use scroll::{Error, Pwrite};
use static_assertions::const_assert;

// https://llvm.org/docs/PDB/DbiStream.html#stream-header
struct_overlay_both!((pub DbiStreamHeaderOverlay, pub DbiStreamHeaderOverlayMut) {
    [0x00] version: u32,
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
const_assert!(DbiStreamHeaderOverlay::size() == 0x40);

// https://llvm.org/docs/PDB/DbiStream.html#optional-debug-header-stream
struct_overlay_both!((pub DbiExtraStreamOverlay, pub DbiExtraStreamOverlayMut) {
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
const_assert!(DbiExtraStreamOverlay::size() == 0x16);

// https://llvm.org/docs/PDB/DbiStream.html#dbi-mod-info-substream
struct_overlay_both!((pub ModInfoOverlay, pub ModInfoOverlayMut) {
    [0x00] unused1: u32,
    [0x04] section: u16,
    [0x06] padding1: [u8; 2],
    [0x08] offset: i32,
    [0x0C] size: i32,
    [0x10] characteristics: u32,
    [0x14] module_index: u16,
    [0x16] padding2: [u8; 2],
    [0x18] data_crc: u32,
    [0x1C] reloc_crc: u32,
    [0x20] flags: u16,
    [0x22] module_sym_stream: u16,
    [0x24] sym_byte_size: u32,
    [0x28] c11_byte_size: u32,
    [0x2C] c13_byte_size: u32,
});
const_assert!(ModInfoOverlay::size() == 0x30);

/// High level abstraction of the DBI stream.
#[derive(Debug, Default, Clone)]
pub struct DbiStream {
    /// The underlying stream information.
    pub stream: Stream,
}

impl DbiStream {
    /// Create a new DbiStream from the underlying Stream.
    pub fn new(stream: Stream) -> Self {
        Self { stream }
    }
    /// Get a read-only DbiStreamHeader.
    pub fn header(&self) -> Option<DbiStreamHeaderOverlay<'_>> {
        DbiStreamHeaderOverlay::new(self.stream.view.as_slice())
    }
    /// Get a mutable DbiStreamHeader.
    pub fn header_mut(&mut self) -> Option<DbiStreamHeaderOverlayMut<'_>> {
        DbiStreamHeaderOverlayMut::new(self.stream.view.as_mut_slice())
    }
    /// This sets the section map descriptor counts to 0
    /// https://github.com/getsentry/pdb/issues/17#issuecomment-2055784958
    /// https://github.com/getsentry/pdb/issues/17#issuecomment-2058271400
    /// https://llvm.org/docs/PDB/DbiStream.html#section-map-substream
    /// Sets "Count" and "LogCount" to 0
    pub fn nop_section_maps(&mut self) -> Result<(), Error> {
        let dbi_header = self
            .header()
            .ok_or_else(|| Error::Custom(format!("Failed to get DbiStreamHeader!")))?;

        let mut offset = (DbiStreamHeaderOverlay::size() as u32
            + (dbi_header.get_mod_info_size() + dbi_header.get_section_contribution_size()))
            as usize;

        // Count = 0
        self.stream
            .view
            .as_mut_slice()
            .gwrite::<u16>(0, &mut offset)?;

        // LogCount = 0
        self.stream
            .view
            .as_mut_slice()
            .gwrite::<u16>(0, &mut offset)?;
        Ok(())
    }
    /// Get the read only extra streams.
    pub fn extra_streams(&self) -> Option<DbiExtraStreamOverlay<'_>> {
        let header = self.header()?;
        // Offset of the DbiExtraStream is after all of these substreams.
        let offset = DbiStreamHeaderOverlay::size()
            + (header.get_mod_info_size()
                + header.get_section_contribution_size()
                + header.get_section_map_size()
                + header.get_source_info_size()
                + header.get_type_server_map_size()
                + header.get_ec_substream_size()) as usize;
        Some(DbiExtraStreamOverlay::new(
            &self.stream.view.as_slice()[offset..],
        )?)
    }
    /// Get a mutable extra streams.
    pub fn extra_streams_mut(&mut self) -> Option<DbiExtraStreamOverlayMut<'_>> {
        let header = self.header()?;
        // Offset of the DbiExtraStream is after all of these substreams.
        let offset = DbiStreamHeaderOverlay::size()
            + (header.get_mod_info_size()
                + header.get_section_contribution_size()
                + header.get_section_map_size()
                + header.get_source_info_size()
                + header.get_type_server_map_size()
                + header.get_ec_substream_size()) as usize;
        Some(DbiExtraStreamOverlayMut::new(
            &mut self.stream.view.as_mut_slice()[offset..],
        )?)
    }
}
