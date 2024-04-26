// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved

use crate::{msf::MsfBigHeaderMut, pagelist::PageList};

/// This is a linear view of a bunch of pages.
#[derive(Debug, Default, Clone)]
pub struct SourceView {
    /// linear sequence of bytes in order.
    pub bytes: Vec<u8>,
    /// The pages associated with this view.
    pub pages: PageList,
}

impl SourceView {
    /// Create a source with its known size.
    pub fn with_size(buff: &[u8], pages: PageList, size: usize) -> Option<SourceView> {
        Self::new(buff, pages).map(|mut view| {
            view.bytes.resize(size, 0);
            view
        })
    }
    /// Creates a linear view of the pages, flush will write them back.
    fn new(buff: &[u8], pages: PageList) -> Option<SourceView> {
        let len = pages.pfns.len() as usize * pages.page_size as usize;
        let mut bytes = Vec::with_capacity(len);
        bytes.resize(len, 0);
        let mut current_offset = 0;
        for pfn in &pages.pfns {
            let page = pfn * pages.page_size;
            if page > buff.len() as u32 {
                return None;
            }
            let slice_end = page + pages.page_size;
            bytes[current_offset as usize..current_offset as usize + pages.page_size as usize]
                .copy_from_slice(&buff[page as usize..slice_end as usize]);
            current_offset += pages.page_size;
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
    /// to the correct pages in "buff". It will also update the page count
    /// inside of the MSF header so that other flushes which add more pages
    /// will work correctly.
    pub fn flush(&mut self, buff: &mut Vec<u8>, header: &mut MsfBigHeaderMut<'_>) {
        // If we need more pages we need to allocate them now.
        if self.bytes.len() > self.pages.len() as usize {
            let cnt_new_pages =
                header.pages_needed_to_store(self.bytes.len() as u32 - self.pages.len());
            let high_pfn = header.get_num_pages();
            for pfn in high_pfn..high_pfn + cnt_new_pages {
                self.pages.push(pfn);
            }
            // Update page count now.
            header.set_num_pages(high_pfn + cnt_new_pages);
            buff.resize(
                buff.len() + (cnt_new_pages * header.get_page_size()) as usize,
                0,
            );
        }
        // Now we need to write bytes back to the file at the correct pages.
        let mut current_offset = 0;
        for pfn in self.pages.pfns.iter() {
            let page_start = *pfn as usize * self.pages.page_size as usize;
            let bytes_to_copy = std::cmp::min(
                self.bytes.len() - current_offset,
                self.pages.page_size as usize,
            );
            buff[page_start..page_start + bytes_to_copy]
                .copy_from_slice(&self.bytes[current_offset..current_offset + bytes_to_copy]);
            current_offset += bytes_to_copy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SourceView;
    use crate::{msf::MsfBigHeaderMut, pagelist::PageList};

    // Basic check to know we map 2 pages.
    #[test]
    fn test_source_view1() {
        let mut buff = Vec::<u8>::new();
        buff.resize(0x5000, 0);
        let mut pages = PageList::new(0x1000);
        pages.push(2);
        pages.push(4);
        let source = SourceView::new(&buff, pages).unwrap();
        // Assert we mapped 2 pages.
        assert_eq!(source.as_slice().len(), 0x2000);
    }

    /// Make sure if we make changes they flush back correctly.
    #[test]
    fn flush_source_view1() {
        let mut buff = Vec::<u8>::new();
        buff.resize(0x5000, 0);
        let mut pages = PageList::new(0x1000);
        pages.push(2);
        pages.push(4);
        let mut source = SourceView::new(&buff, pages).unwrap();
        source.as_mut_slice()[0..0x1000].fill(0x69);
        source.as_mut_slice()[0x1000..0x2000].fill(0x42);
        let mut header_bytes = Vec::<u8>::new();
        header_bytes.resize(0x1000, 0);
        let mut header = MsfBigHeaderMut::new(&mut header_bytes).unwrap();
        header.set_page_size(0x1000);
        header.set_num_pages(5);
        source.flush(&mut buff, &mut header);
        // Make sure the flush actually works.
        assert!(buff[0x2000..0x3000].iter().all(|&e| e == 0x69));
        assert!(buff[0x4000..0x5000].iter().all(|&e| e == 0x42));
    }

    /// Expand the mapping and make sure that it flushes
    /// back and expands the vector we flush to.
    #[test]
    fn flush_source_view2() {
        let mut buff = Vec::<u8>::new();
        buff.resize(0x5000, 0);
        let mut pages = PageList::new(0x1000);
        pages.push(2);
        pages.push(4);
        let mut source = SourceView::new(&buff, pages).unwrap();
        source.as_mut_slice()[0..0x1000].fill(0x69);
        source.as_mut_slice()[0x1000..0x2000].fill(0x42);
        source.bytes.resize(source.bytes.len() + 0x500, 0xFF);

        let mut header_bytes = Vec::<u8>::new();
        header_bytes.resize(0x1000, 0);
        let mut header = MsfBigHeaderMut::new(&mut header_bytes).unwrap();

        header.set_page_size(0x1000);
        header.set_num_pages(5);
        source.flush(&mut buff, &mut header);

        assert_eq!(header.get_num_pages(), 6);
        // Make sure the flush actually works.
        assert!(buff[0x2000..0x3000].iter().all(|&e| e == 0x69));
        assert!(buff[0x4000..0x5000].iter().all(|&e| e == 0x42));
    }
}
