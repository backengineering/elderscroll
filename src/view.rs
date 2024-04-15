// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use crate::pagelist::PageList;

/// This is a linear view of a bunch of pages.
#[derive(Debug, Default, Clone)]
pub struct SourceView {
    /// linear sequence of bytes in order.
    bytes: Vec<u8>,
    /// The pages associated with this view.
    pages: PageList,
}

impl SourceView {
    /// Creates a linear view of the pages, flush will write them back.
    pub fn new(buff: &[u8], pages: PageList) -> Option<SourceView> {
        let len = pages.source_slices.iter().fold(0, |acc, s| acc + s.size);
        let mut bytes = Vec::with_capacity(len);
        bytes.resize(len, 0);
        let mut current_offset = 0;
        for slice in &pages.source_slices {
            if slice.offset as usize + slice.size > buff.len() {
                return None;
            }
            let slice_end = slice.offset as usize + slice.size;
            bytes[current_offset..current_offset + slice.size]
                .copy_from_slice(&buff[slice.offset as usize..slice_end]);
            current_offset += slice.size;
        }
        Some(SourceView { bytes, pages })
    }
    /// Get a read-only slice of the mapping.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
    /// Get a mutable reference to the mapping.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
    /// This function will flush the internal mapping back
    /// to the correct pages in "buff".
    pub fn flush(&mut self, buff: &mut [u8]) {
        unimplemented!()
    }
}
