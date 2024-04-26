// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved

use crate::{
    msf::{MsfBigHeader, MsfBigHeaderMut},
    pagelist::PageList,
    view::SourceView,
};
use scroll::{Error, Pread, Pwrite};

/// This is the constant for invalid stream indices.
pub const INVALID_STREAM_INDEX: u16 = 0xFFFF;
pub const INVALID_STREAM_SIZE: u32 = u32::MAX;
pub const DBI_STREAM_INDEX: usize = 3;

/// Abstraction of the stream itself.
#[derive(Debug, Default, Clone)]
pub struct Stream {
    /// Original size of the stream when parsing it. Do not use this or touch it.
    /// Consider it as READ ONLY.
    pub original_stream_size: u32,
    /// Linear mapping of the stream.
    pub view: SourceView,
}

/// Abstraction of the stream directory.
#[derive(Debug, Default, Clone)]
pub struct StreamDirectory {
    /// Abstract lifted view of every stream, this stream is sorted.
    /// So you can index into it with the stream index.
    pub streams: Vec<Stream>,
    /// Linear mapping of the pages used for the StreamDirectory.
    pub view: SourceView,
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
                original_stream_size: buff.gread::<u32>(&mut offset)?,
                ..Default::default()
            });
        }
        // Read the pages for each stream.
        for stream in streams.iter_mut() {
            // Some streams have no size so there are no PFN's to read.
            if stream.original_stream_size != INVALID_STREAM_SIZE {
                let num_pages = header.pages_needed_to_store(stream.original_stream_size);
                let mut pages = PageList::new(header.get_page_size());
                for _ in 0..num_pages {
                    pages.push(buff.gread::<u32>(&mut offset)?);
                }
                // Parse the stream out of the PDB file now.
                stream.view =
                    SourceView::with_size(bytes, pages, stream.original_stream_size as usize)
                        .ok_or_else(|| {
                            Error::Custom("Failed to create view for streams!".to_string())
                        })?;
            }
        }
        Ok(Self { view, streams })
    }
    /// Flush directory back into the file.
    #[inline(always)]
    pub fn flush(
        &mut self,
        buff: &mut Vec<u8>,
        header: &mut MsfBigHeaderMut<'_>,
    ) -> Result<(), Error> {
        // Compute the size of the StreamDirectory
        // NumberOfStreams is 4 bytes.
        let mut stream_directory_size = 4u32;
        // Each stream needs 4 bytes for its len.
        stream_directory_size += self.streams.len() as u32 * 4;
        // Compute how many PFN's there are for all streams.
        for stream in self.streams.iter_mut() {
            // Flush stream bytes back now.
            stream.view.flush(buff, header);
            // DWORD for each pfn.
            stream_directory_size += stream.view.pages.pfns.len() as u32 * 4;
        }
        // Update the size of the stream directory.
        header.set_stream_dir_size(stream_directory_size);
        // Write the stream directory into the view now.
        let mut offset = 0;
        // Resize the mapping of the StreamDirectory.
        self.view.bytes.resize(stream_directory_size as usize, 0);
        // Write the number of streams (NumStreams)
        self.view
            .bytes
            .gwrite::<u32>(self.streams.len() as u32, &mut offset)?;
        // Write each streams size now.
        for stream in self.streams.iter() {
            if stream.view.bytes.is_empty() {
                self.view.bytes.gwrite::<u32>(u32::MAX, &mut offset)?;
            } else {
                self.view
                    .bytes
                    .gwrite::<u32>(stream.view.bytes.len() as u32, &mut offset)?;
            }
        }
        // Write each streams pfn now.
        for stream in self.streams.iter() {
            for pfn in stream.view.pages.pfns.iter() {
                self.view.bytes.gwrite::<u32>(*pfn, &mut offset)?;
            }
        }
        // Flush the stream directory back to the file.
        self.view.flush(buff, header);
        // Finally we need to update the StreamDirectoryMap page.
        let stream_block_map = &mut buff[header.stream_block_map()..];
        // Zero the map page for debug purposes.
        stream_block_map[0..header.get_page_size() as usize].fill(0);
        // Write the PFN's used by the StreamDirectory
        let mut offset = 0;
        for pfn in self.view.pages.pfns.iter() {
            stream_block_map.gwrite::<u32>(*pfn, &mut offset)?;
        }
        Ok(())
    }
}
