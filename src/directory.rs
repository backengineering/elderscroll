// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use crate::pagelist::PageList;

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
    pub pages: PageList,
}