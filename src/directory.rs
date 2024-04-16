// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use crate::{
    msf::{MsfBigHeader, MsfBigHeaderMut},
    pagelist::PageList,
    view::SourceView,
};
use scroll::{Error, Pread};

/// This is the constant for invalid stream indices.
pub const INVALID_STREAM_INDEX: u16 = 0xFFFF;
pub const INVALID_STREAM_SIZE: u32 = u32::MAX;
pub const DBI_STREAM_INDEX: StreamIndex = StreamIndex(3);

/// Stream index abstraction which offers a "is_valid"
/// To ensure that the stream index is OK.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StreamIndex(u16);

impl StreamIndex {
    /// Returns true if stream index is not INVALID_STREAM_INDEX
    pub fn is_valid(&self) -> bool {
        self.0 != INVALID_STREAM_INDEX
    }
}

/// Abstraction of the stream itself.
#[derive(Debug, Default, Clone)]
pub struct Stream {
    /// Byte size of stream.
    pub size: u32,
    /// Linear mapping of the stream.
    pub view: SourceView,
}

#[derive(Debug, Default, Clone)]
pub struct StreamDirectory {
    /// Abstract lifted view of every stream, sorted by StreamIndex.
    streams: Vec<Stream>,
    /// Linear mapping of the pages used for the StreamDirectory.
    view: SourceView,
}

impl StreamDirectory {
    /// Lift entire stream directory table and all streams as well.
    pub fn new(bytes: &[u8], view: SourceView, header: &MsfBigHeader<'_>) -> Result<Self, Error> {
        let buff = view.as_slice();
        let mut offset = 0;
        // Read the number of streams.
        let num_streams = buff.gread::<u32>(&mut offset)?;
        let mut streams = Vec::<Stream>::new();
        // Read all of the sizes for each stream.
        for _ in 0..num_streams {
            streams.push(Stream {
                size: buff.gread::<u32>(&mut offset)?,
                ..Default::default()
            });
        }
        // Read the pages for each stream.
        for stream in streams.iter_mut() {
            // Some streams have no size so there are no PFN's to read.
            if stream.size != INVALID_STREAM_SIZE {
                let num_pages = header.pages_needed_to_store(stream.size);
                let mut pages = PageList::new(header.get_page_size());
                for _ in 0..num_pages {
                    pages.push(buff.gread::<u32>(&mut offset)?);
                }
                // Parse the stream out of the PDB file now.
                stream.view = SourceView::new(bytes, pages).ok_or_else(|| {
                    Error::Custom("Failed to create view for streams!".to_string())
                })?;
            }
        }
        Ok(Self { view, streams })
    }
    /// Flush directory back into the file.
    #[inline(always)]
    pub fn flush(&mut self, buff: &mut [u8], header: &mut MsfBigHeaderMut<'_>) {
        self.view.flush(buff, header);
    }
    /// Returns the number of streams.
    #[inline(always)]
    pub fn number_of_streams(&self) -> u16 {
        self.streams.len() as u16
    }
    /// Getter for a stream with a stream index check.
    #[inline(always)]
    pub fn get_stream(&self, idx: StreamIndex) -> Option<Stream> {
        if idx.is_valid() {
            Some(self.streams[idx.0 as usize].clone())
        } else {
            None
        }
    }
    /// Setter for a stream with a stream index check.
    #[inline(always)]
    pub fn set_stream(&mut self, idx: StreamIndex, stream: Stream) -> Option<()> {
        if idx.is_valid() {
            self.streams[idx.0 as usize] = stream;
            Some(())
        } else {
            None
        }
    }
}
