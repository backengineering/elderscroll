// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use crate::msf::{MsfBigHeader, PageNumber};

/// Represents an offset + size of the source file.
///
/// The multi-stream file implementation (used by `pdb::PDB`) determines which byte ranges it needs
/// to satisfy its requests, and it describes those requests as a `&[SourceSlice]`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SourceSlice {
    /// Offset into the source file.
    pub offset: u32,
    /// Size of the slice.
    pub size: u32,
}

/// List of pages used by a stream
#[derive(Debug, Default, Clone)]
pub struct PageList {
    /// Valid values are 512, 1024, 2048, and 4096
    pub page_size: u32,
    /// All of the page ranges.
    pub pfns: Vec<PageNumber>,
}

impl PageList {
    /// Create a new PageList for a given page size.
    pub fn new(page_size: u32) -> Self {
        Self {
            page_size,
            pfns: Vec::new(),
        }
    }
    /// Add a page to the PageList.
    #[inline(always)]
    pub fn push(&mut self, page: PageNumber) {
        self.pfns.push(page);
    }
    /// Return the total length of this PageList.
    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.pfns.len() as u32 * self.page_size
    }
}
