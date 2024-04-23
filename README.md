# Elderscroll

This is a small PDB rewriting library. This code will (re)create the OMAP streams so that moved ranges of code can still map back to their original places in the PDB. The PDB is so old that i refer to it as an elderscroll.

This library will only work for PDB 7.0 files (aka large MSF files).

**_This project is heavily pasted from pdb-rs_**

### Limits

This project will only change the OMAP streams in the PDB file. These streams are not used by every PDB parser out there.

***You must use the old windbg to view the changes we make to the PDB with this library.***

### PDB Details

The PDB file format is actually a file system within a file. The format is an "MSF" (Multi Stream File). Just know that a single file can contain multiple "streams". Each of these streams contains bytes.

Please use the following links to understand more about the PDB and OMAP:

- https://chromium.googlesource.com/chromiumos/platform/google-breakpad/+/refs/heads/release-R41-6680.B/src/common/windows/omap.cc
- https://github.com/getsentry/pdb/issues/17
- https://llvm.org/docs/PDB/
- https://llvm.org/docs/PDB/MsfFile.html#file-layout
- https://llvm.org/docs/PDB/DbiStream.html
- https://learn.microsoft.com/en-us/windows/win32/api/dbghelp/ns-dbghelp-omap#remarks
- https://github.com/getsentry/pdb/pull/35

This library does not care about anything in the PDB that is not related to (re)creating the OMAP streams. If you want to parse a PDB use the `pdb-rs` crate. Maybe one day we can merge some of my code into `pdb-rs`.

### Moving code and OMAP

The PDB contains a method for us to describe how ranges of bytes might have been moved for instrumentation or transformations purposes. This method is called the OMAP stream(s) which are a pair of streams that map ranges to-and-from transformed binaries and original binaries.

```
                          ┌──────────────────────┐
                ┌─────────│     omap_to_src      │◀────────┐
                │         └──────────────────────┘         │
                ▼                                          │
┌──────────────────────────────┐           ┌──────────────────────────────┐
│         Original PE          │           │        Rearranged PE         │
├──────────────────────────────┤           ├──────────────────────────────┤
│   original_section_headers   │           │       section_headers        │
└──────────────────────────────┘           └──────────────────────────────┘
                │                                          ▲
                │        ┌──────────────────────┐          │
                └───────▶│    omap_from_src     │──────────┘
                         └──────────────────────┘
```

These OMAP streams are defined inside of the "extra streams" header inside of the DBI stream. Most PDB files will not have OMAP streams because their layouts have not been changed. In that case no OMAP streams exist and new ones must be created.

However, its important to note that OMAP streams are NOT the only component of the PDB involved with OMAP translation. There are two streams that contain section headers, one for the original binary and one for the new binary. These streams are also defined in the "extra streams".

There is also a "sections map" sub-stream that is used in address translation. I just zero it out, if that becomes a problem we will [spin the block](https://www.urbandictionary.com/define.php?term=Spin%20the%20block).

```rust
struct_overlay_both!((pub DbiExtraStream, pub DbiExtraStreamMut) {
    [0x00] fpo_data: u16,
    [0x02] exception_data: u16,
    [0x04] fixup_data: u16,
    [0x06] omap_to_src: u16,              // <--- OMAP to stream
    [0x08] omap_from_src: u16,            // <--- OMAP from stream
    [0x0A] section_headers: u16,          // <--- New section header stream
    [0x0C] unknown1: u16,
    [0x0E] xdata: u16,
    [0x10] pdata: u16,
    [0x12] fpo2_data: u16,
    [0x14] original_section_headers: u16, // <--- Original headers stream
});
```