# File Tool

[![CI](https://github.com/jeremyje/filetools/actions/workflows/ci.yaml/badge.svg)](https://github.com/jeremyje/filetools/actions/workflows/ci.yaml)
![Crates.io Version](https://img.shields.io/crates/v/filetool)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://github.com/jeremyje/filetools/blob/master/LICENSE)
[![GitHub release](https://img.shields.io/github/release-pre/jeremyje/filetools.svg)](https://github.com/jeremyje/filetools/releases)

```text
A tool to manage and cleanup files on your hard drive.

Usage: filetool [OPTIONS] <COMMAND>

Commands:
  checksum
          Calculates checksums (xxhash3-64bit) of files in selected directories
  clean-empty-directory
          Removes directories that do not contain any files
  duplicate
          Finds duplicate files and conditionally deletes them
  rmlist
          Delete files from file lists
  similar-name
          List files with similar file names
  help
          Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
  -h, --help
          Print help
  -V, --version
          Print version
```
