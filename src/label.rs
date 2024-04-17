// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

/// These bytes are above every single symbol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SymbolHeader {
    /// Symbol size in bytes.
    pub symbol_size: u16,
    /// Symbol kind.
    pub symbol_kind: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabelSymbol {
    /// Internally set to [`S_LABEL32`]
    ty: u16,
    /// Offset into section.
    pub offset: u32,
    /// Section number in section header stream.
    pub section: u16,
    /// Name of the label.
    pub name: String,
}

/// Defaults ty to `S_LABEL32`
impl Default for LabelSymbol {
    fn default() -> Self {
        Self {
            ty: S_LABEL32,
            offset: Default::default(),
            section: Default::default(),
            name: Default::default(),
        }
    }
}

/// Label symbol kind.
const S_LABEL32: u16 = 0x1105;
