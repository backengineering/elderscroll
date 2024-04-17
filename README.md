# Elderscroll

This is a small PDB rewriting library. This library is a ground up rewrite of `pdb-rs`. It will not parse as much information as `pdb-rs` but allows for editing MSF streams.

This library will only work for PDB 7.0 files (aka large MSF files).

### PDB Details

The PDB file format is actually a file system within a file. The format is an "MSF" (Multi Stream File). Just know that a single file can contain multiple "streams". Each of these streams contains bytes. These streams are *kinda* documented by LLVM. If you are interested in the format of any of the streams in the PDB please refer to llvm docs:

- https://llvm.org/docs/PDB/