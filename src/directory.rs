// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use crate::{msf::MsfBigHeader, pagelist::PageList, view::SourceView};
use scroll::{Error, Pread};

/// This is the constant for invalid stream indices.
pub const INVALID_STREAM_INDEX: u16 = 0xFFFF;

/// Stream index abstraction which offers a "is_valid"
/// To ensure that the stream index is OK.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StreamIndex(u16);

impl StreamIndex {
    /// Returns true if stream index is not INVALID_STREAM_INDEX
    pub fn is_valid(&self) -> bool {
        self.0 == INVALID_STREAM_INDEX
    }
}

/// Abstraction of the stream itself.
#[derive(Debug, Default, Clone)]
pub struct StreamDetails {
    /// Byte size of stream.
    pub size: u32,
    /// Pages used by this stream.
    pub pages: PageList,
}

#[derive(Debug, Default, Clone)]
pub struct StreamDirectory {
    /// Pages used by the stream directory.
    pub view: SourceView,
}

impl StreamDirectory {
    pub fn new(view: SourceView, header: &MsfBigHeader<'_>) -> Result<Self, Error> {
        let buff = view.as_slice();
        let mut offset = 0;
        // Read the number of streams.
        let num_streams = buff.gread::<u32>(&mut offset)?;
        let mut stream_sizes = Vec::new();
        // Read all of the sizes for each stream.
        for _ in 0..num_streams {
            stream_sizes.push(buff.gread::<u32>(&mut offset)?);
        }
        // Read the pages for each stream.
        Ok(Self { view })
    }
}
