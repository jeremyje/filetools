# File Tool

[![CI](https://github.com/jeremyje/filetools/actions/workflows/ci.yaml/badge.svg)](https://github.com/jeremyje/filetools/actions/workflows/ci.yaml)
![Crates.io Version](https://img.shields.io/crates/v/filetool)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://github.com/jeremyje/filetools/blob/master/LICENSE)
[![GitHub release](https://img.shields.io/github/release-pre/jeremyje/filetools.svg)](https://github.com/jeremyje/filetools/releases)

```text
A tool to manage and cleanup files on your hard drive.

Usage: filetool [OPTIONS] <COMMAND>

Commands:
  canonical
          Renames files to standard names. Typically this is renaming unusual file extensions
  checksum
          Calculates checksums (xxhash3-64bit) of files in selected directories
  clean-empty-directory
          Removes directories that do not contain any files
  clean-filename
          List files with similar file names
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

### `canonical`

```text
Renames files to standard names. Typically this is renaming unusual file extensions

Usage: filetool canonical [OPTIONS]

Options:
      --path <PATH>
          List of paths to scan for files with non-canonical extensions (e.g. .jpeg → .jpg, .mp4 → .m4v) [default: .]
      --dry-run <DRY_RUN>
          When enabled (default), reports files that would be renamed without making changes. Set to false to actually rename files [default: true] [possible values: true, false]
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
  -h, --help
          Print help
```

### `checksum`

```text
Calculates checksums (xxhash3-64bit) of files in selected directories

Usage: filetool checksum [OPTIONS]

Options:
      --path <PATH>
          List of directories to scan for files that will have their checksums calculated [default: .]
      --output <OUTPUT>
          Output file where the checksum database will be written to [default: checksums.txt]
      --checksum-threads <CHECKSUM_THREADS>
          Number of threads for calculating checksums [default: 2]
      --checksum-checkpoint-interval <CHECKSUM_CHECKPOINT_INTERVAL>
          How often to save the checksum database to disk during scanning. Shorter intervals reduce data loss if the process is interrupted [default: 30s]
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
  -h, --help
          Print help
```

### `clean-empty-directory`

```text
Removes directories that do not contain any files

Usage: filetool clean-empty-directory [OPTIONS]

Options:
      --path <PATH>
          List of directories that will be scanned to be removed if empty [default: .]
      --dry-run <DRY_RUN>
          When enabled (default), reports empty directories without deleting them. Set to false to actually delete empty directories [default: true] [possible values: true, false]
      --force
          Force deletion of directories even when the read-only bit is set
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
  -h, --help
          Print help
```

### `clean-filename`

```text
List files with similar file names

Usage: filetool clean-filename [OPTIONS]

Options:
      --path <PATH>
          List of directories that will be scanned for files to be renamed [default: .]
      --dry-run <DRY_RUN>
          When enabled (default), reports files that would be renamed without making changes. Set to false to rename files with unusual naming patterns such as URL-encoded strings [default: true] [possible values: true, false]
      --overwrite
          Overwrite existing files if present
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
  -h, --help
          Print help
```

### `duplicate`

```text
Finds duplicate files and conditionally deletes them

Usage: filetool duplicate [OPTIONS]

Options:
      --path <PATH>
          List of paths to scan for duplicate files [default: .]
      --min-size <MIN_SIZE>
          Minimum file size in bytes to include in the duplicate scan. Files smaller than this are ignored [default: 0]
      --delete-pattern <DELETE_PATTERN>
          Glob patterns matched against file paths to select duplicates for deletion. A file is deleted when its path matches any delete pattern and none of the keep patterns. Example: --delete-pattern='**/trash/**' [default: ""]
      --keep-pattern <KEEP_PATTERN>
          Glob patterns matched against file paths to protect files from deletion, even if they also match a delete pattern. Example: --keep-pattern='**/important/**' [default: ""]
      --dry-run <DRY_RUN>
          When enabled (default), reports which files would be deleted without removing them. Set to false to delete duplicate files matching `--delete-pattern` [default: true] [possible values: true, false]
      --output <OUTPUT>
          Path where the duplicate report will be written. Supports .html and .csv formats [default: duplicates.html]
      --overwrite <OVERWRITE>
          Overwrite the output report file if it already exists [default: true] [possible values: true, false]
      --db <DB>
          Path where the checksum database is stored. Reusing this file on subsequent runs avoids re-hashing unchanged files [default: checksums.txt]
      --rmlist <RMLIST>
          Path where the list of duplicate files selected for deletion is recorded, one path per line [default: rmlist.txt]
      --checksum-threads <CHECKSUM_THREADS>
          Number of threads for calculating checksums [default: 2]
      --checksum-checkpoint-interval <CHECKSUM_CHECKPOINT_INTERVAL>
          How often to save the checksum database to disk during scanning. Shorter intervals reduce data loss if the process is interrupted [default: 30s]
      --force
          Force deletion of files when the read-only bit is set
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
  -h, --help
          Print help
```

### `rmlist`

```text
Delete files from file lists

Usage: filetool rmlist [OPTIONS]

Options:
      --path <PATH>
          List of rmlist files to process. Each file must contain one file path per line [default: .]
      --dry-run <DRY_RUN>
          When enabled (default), lists files that would be deleted without actually removing them. Set to false to permanently delete the listed files [default: true] [possible values: true, false]
      --force
          Force deletion of files when the read-only bit is set
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
  -h, --help
          Print help
```

### `similar-name`

```text
List files with similar file names

Usage: filetool similar-name [OPTIONS]

Options:
      --path <PATH>
          List of directory paths to scan for similarly named files [default: .]
      --clear-tokens <CLEAR_TOKENS>
          Substrings that are ignored in file names to determine if a file is similar. Example: --clear-tokens=(1) will match "image.jpg" and "image (1).jpg" since the space and "(1)" are ignored [default: ""]
      --min-size <MIN_SIZE>
          Minimum file size in bytes to include in the scan. Files smaller than this are ignored [default: 0]
      --include-size <INCLUDE_SIZE>
          Include file sizes in the output when reporting similar file groups [default: false] [possible values: true, false]
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
  -h, --help
          Print help
```

## Installation

Linux

`curl -o filetool -O -L https://github.com/jeremyje/filetools/releases/download/v0.6.5/filetool; chmod +x filetool`

Windows

`(New-Object System.Net.WebClient).DownloadFile("https://github.com/jeremyje/filetools/releases/download/v0.6.5/filetool.exe", "filetool.exe")`
