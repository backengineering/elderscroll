# Elderscroll

This is a small PDB rewriting library. This code will (re)create the OMAP streams so that moved ranges of code can still map back to their original places in the PDB. The PDB is so old that i refer to it as an elderscroll.

This library will only work for PDB 7.0 files (aka large MSF files).

***This project is heavily pasted from pdb-rs***

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

This library does not care about anything the PDB that is not related to (re)creating the OMAP streams. If you want to parse a PDB use the `pdb-rs` crate.