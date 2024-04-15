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
    pub offset: u64,
    /// Size of the slice.
    pub size: usize,
}

/// List of pages used by a stream
#[derive(Debug, Default, Clone)]
pub struct PageList {
    /// Valid values are 512, 1024, 2048, and 4096
    pub page_size: usize,
    /// All of the page ranges.
    pub source_slices: Vec<SourceSlice>,
}

impl PageList {
    /// Create a new PageList for a given page size.
    pub fn new(page_size: usize) -> Self {
        Self {
            page_size,
            source_slices: Vec::new(),
        }
    }

    /// Return all of the page numbers in order. This i use
    /// to write data into the pages themselves.
    pub fn pfns(&self, header: &MsfBigHeader<'_>) -> Vec<PageNumber> {
        let mut result = Vec::new();
        for slice in self.source_slices() {
            // Compute the size of the slice in pages.
            let num_pages = header
                .pages_needed_to_store(((slice.offset + slice.size as u64) - slice.offset) as u32);
            // For each page starting at the first push the frame number.
            let pfn_start = slice.offset as u32 / header.get_page_size();
            for pfn in pfn_start..pfn_start + num_pages {
                result.push(pfn);
            }
        }
        result
    }

    /// Add a page to the PageList.
    pub fn push(&mut self, page: PageNumber) {
        self.source_slices.push(SourceSlice {
            offset: (self.page_size as u64) * u64::from(page),
            size: self.page_size,
        });
    }

    /// Return the total length of this PageList.
    pub fn len(&self) -> usize {
        self.source_slices.iter().fold(0, |acc, s| acc + s.size)
    }

    /// Return a slice of SourceSlices.
    pub fn source_slices(&self) -> &[SourceSlice] {
        self.source_slices.as_slice()
    }
}
