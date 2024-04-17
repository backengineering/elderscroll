// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use scroll::{Error, Pwrite};
use std::{cmp::Ordering, collections::BTreeSet};

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
pub struct OmapStream(pub BTreeSet<OmapEntry>);

impl OmapStream {
    /// Convert the Omap stream to bytes.
    pub fn to_vec(&self) -> Result<Vec<u8>, Error> {
        let mut buff = vec![0u8; self.0.len() * 8];
        let mut offset = 0;
        for entry in self.0.iter() {
            buff.gwrite(entry.0, &mut offset)?;
            buff.gwrite(entry.1, &mut offset)?;
        }
        Ok(buff)
    }
}