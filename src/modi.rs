// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Read: https://llvm.org/docs/PDB/ModiStream.html

use scroll::{Error, Pwrite};
use static_assertions::const_assert;

use crate::{
    directory::{Stream, INVALID_STREAM_SIZE},
    label::LabelSymbol,
    pagelist::PageList,
    struct_overlay_both,
    view::SourceView,
};

/// Unknown. In practice only the value of 4 has been observed.
/// It is hypothesized that this value corresponds to the set of CV_SIGNATURE_xx
/// defines in cvinfo.h, with the value of 4 meaning that this module has C13 line
/// information (as opposed to C11 line information).
const MODI_STREAM_SIGNATURE: u32 = 4;

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

// https://llvm.org/docs/PDB/ModiStream.html#stream-layout
#[derive(Debug, Clone)]
pub struct ModiStream {
    /// Internal stream
    pub stream: Stream,
    /// Internally used to know the offset of symbols
    /// so that we just append new ones. Do not use this to write signature!
    offset: usize,
}

impl ModiStream {
    /// Create a new module stream given page size. This will build
    /// a new stream internally.
    pub fn new(page_size: u32) -> Result<Self, Error> {
        let mut stream = Stream {
            original_stream_size: INVALID_STREAM_SIZE,
            view: SourceView {
                // Add 4 bytes for the signature.
                // Add 4 bytes of zeros for "GlobalRefsSize" and "GlobalRefs"
                // https://llvm.org/docs/PDB/ModiStream.html#the-codeview-symbol-substrea
                bytes: vec![0u8, 8],
                pages: PageList::new(page_size),
            },
        };
        // Write signature into buffer.
        stream.view.bytes.resize(8, 0);
        stream.view.bytes.pwrite::<u32>(MODI_STREAM_SIGNATURE, 0)?;
        Ok(Self {
            stream,
            offset: 4, // 4 byte offset skipping signature.
        })
    }
    /// Add a label to the [`ModiStream`]
    #[inline(always)]
    pub fn add_label(&mut self, label: LabelSymbol) -> Result<(), Error> {
        // Extend stream size. +4 for the label symbol header.
        self.stream
            .view
            .bytes
            .resize(self.stream.view.bytes.len() + label.size() + 4, 0);
        // Write the label symbol (including its symbol header.)
        self.stream
            .view
            .bytes
            .gwrite(label, &mut self.offset)
            .map(|_| ())
    }
}
