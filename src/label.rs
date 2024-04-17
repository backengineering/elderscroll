// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

use scroll::{ctx, Endian, Pwrite};

/// Label symbol used to name a location within a PE.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LabelSymbol {
    /// Offset into section.
    pub offset: u32,
    /// Section number in section header stream.
    pub section: u16,
    /// [`ProcedureFlags`], ignore this value for now.
    pub flags: u8,
    /// Name of the label.
    pub name: String,
}

impl LabelSymbol {
    /// Gets the size of the symbol including the name and null terminator.
    pub fn size(&self) -> usize {
        // offset + section + flags + string length + null term.
        4 + 2 + 1 + self.name.len() + 1 + 2
    }
}

/// Label symbol kind.
const S_LABEL32: u16 = 0x1105;

impl ctx::TryIntoCtx<Endian> for LabelSymbol {
    type Error = scroll::Error;
    /// This will write a label and its symbol header into the buffer.
    fn try_into_ctx(self, buff: &mut [u8], le: Endian) -> Result<usize, Self::Error> {
        if buff.len() < 2 {
            return Err((scroll::Error::Custom(format!("Buffer too small!"))).into());
        }
        let mut offset = 0;
        // https://llvm.org/docs/PDB/CodeViewTypes.html#leaf-records
        // All symbols have a header with a size and type.
        // Write the length of the symbol.
        buff.gwrite_with::<u16>(self.size() as u16, &mut offset, le)?;
        // Write the type of the symbol.
        buff.gwrite_with::<u16>(S_LABEL32, &mut offset, le)?;
        // Write the symbol itself now.
        buff.gwrite_with::<u32>(self.offset, &mut offset, le)?;
        buff.gwrite_with::<u16>(self.section, &mut offset, le)?;
        buff.gwrite_with::<u8>(self.flags, &mut offset, le)?;
        let mut string_bytes = self.name.as_bytes().to_vec();
        string_bytes.push(0); // rusts "as_bytes" doesnt include nul.
        buff[offset..offset + string_bytes.len()].copy_from_slice(&string_bytes);
        Ok(offset + string_bytes.len())
    }
}

#[cfg(test)]
mod tests {
    use super::LabelSymbol;
    use scroll::Pwrite;
    /// Test to make sure that the symbol information is all correct.
    #[test]
    fn label_symbol_test() {
        let mut symbol = LabelSymbol::default();
        symbol.name = format!("HelloWorld");
        let mut buff = vec![0u8; 0x100];
        buff.pwrite(symbol, 0).unwrap();
        println!("{:X?}", buff);
    }
}
