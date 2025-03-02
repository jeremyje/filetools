# Development Notes

## Old Readme

Coming Soon

* Binary releases for Windows, Linux, and MacOs

Unique Bugs

* Improve Report
  * Sort Items by size, descending
  * File Size Function
* Actually support different hash algorithms.

Similar

* Use sharded multiwalk and delete hold multiwalk function since it's prone to race conditions.
* similar tests are dead locking.
* Similar cannot handle multiple paths yet because of race-condition in multiwalk acting on 1 map.

## Rust Documentation

<https://jake-shadle.github.io/xwin/#using-x86-64-pc-windows-gnu>
<https://blog.burntsushi.net/rust-error-handling/>
<https://rust-cli.github.io/book/tutorial/cli-args.html>

## Cross Compile

```bash
sudo apt-get install -y mingw-w64 g++-aarch64-linux-gnu libc6-dev-arm64-cross libssl-dev
rustup target add x86_64-pc-windows-gnu
cargo build --target x86_64-pc-windows-gnu
rustup toolchain install stable-aarch64-unknown-linux-gnu
cargo install cargo-tarpaulin
```

## Resources

* <https://doc.rust-lang.org/rust-by-example/index.html>
* <https://rust-cli.github.io/book/tutorial/index.html>
* <https://rust-lang-nursery.github.io/rust-cookbook/cryptography/hashing.html>

## Duplicates

Fast duplicate detection using Rust.

1. Scan devices based on a thread per device for scanning so that all devices are scanned at the same time.
1. Heuristics for detecting duplicate files.
   1. Files that are 4KiB or less.
   1. Larger files with the same size.
   1. Use a lean hash algorithm such as CRC-32.
1. Concurrent read I/O and heavy core use for hashing.

## Clean Filename

Cleans up file name

Usage of G:\tools\filename.exe:
  -dry_run
        Only report changes, don't change file names. (default true)
  -path string
        Path of directory tree to sanitize file names. (default ".")
  -plusreplace
        Replace + with space.
  -strip string
        Strings to remove.
  -verbose
        Enable verbose log output

Usage of G:\tools\org.exe:
  -dir string
        Directory to organize one level depth. (default ".")
  -dry_run
        Report only, do not move (default true)

PS G:\tools> .\similar.exe --help
Usage of G:\tools\similar.exe:
  -clear string
        Clear tokens
  -include_size
        Show size of file (default true)
  -min_size int
        Minimum file size (in bytes) to include file.
  -path string
        Path of directory tree to scan for duplicates

PS G:\tools> .\unique.exe --help
Usage of G:\tools\unique.exe:
  -delete string
        CSV list of directories may contain dup files that can be deleted.
  -dry_run
        Actually deletes when set to false. (default true)
  -min_size int
        Minimum file size (in bytes) to include file.
  -output string
        Path of output report file. (default "duplicates.html")
  -path string
        Path of directory tree to scan for duplicates.
  -verbose
        Enable verbose log output.
