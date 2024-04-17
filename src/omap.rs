// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use crate::directory::Stream;
use scroll::{Error, Pread};
use std::cmp::Ordering;

/// (Source -> Target)
/// Entries are used to map code from one layout to another.
/// Refer to the read of this project for OMAP info.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd)]
pub struct OmapEntry(pub u32, pub u32);

impl Ord for OmapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

/// OMAP stream, used for both "to" and "from" mappings.
#[derive(Debug, Default, Clone)]
pub struct OmapStream(Vec<OmapEntry>);

impl OmapStream {
    /// Convert the Omap stream to bytes.
    pub fn new(stream: Stream) -> Result<Self, Error> {
        let mut set = Vec::<OmapEntry>::new();
        let mut addr = 0;
        let mut offset = 0;
        loop {
            let source = stream.view.bytes.gread::<u32>(&mut offset)?;
            let target = stream.view.bytes.gread::<u32>(&mut offset)?;
            if addr < source {
                break;
            }
            addr = source;
            set.push(OmapEntry(source, target));
        }
        Ok(Self(set))
    }

    /// Look up `source_address` to yield a target address.
    /// if no OMAP range exists for `source_address` it just returns `source_address`.
    pub fn translate(&self, source_address: u32) -> u32 {
        let index = match self.0.binary_search_by_key(&source_address, |r| r.0) {
            Ok(i) => i,
            Err(0) => return source_address,
            Err(i) => i - 1,
        };
        let record = &self.0[index];
        if record.1 == 0 {
            return source_address;
        }
        (source_address - record.0) + record.1
    }
}
