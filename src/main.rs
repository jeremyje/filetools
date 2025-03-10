// Copyright 2024 Jeremy Edwards
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod canonical;
mod checksum;
mod clean_empty_directory;
mod clean_filename;
mod common;
mod duplicate;
mod rmlist;
mod similar_name;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

#[macro_use(concat_string)]
extern crate concat_string;

// https://docs.rs/clap/latest/clap/_derive/_tutorial/chapter_1/index.html
#[derive(Parser)]
#[command(name = "filetool")]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Subcommand)]
enum Commands {
    /// Renames files to standard names. Typically this is renaming unusual file extensions.
    Canonical(canonical::Args),
    /// Calculates checksums (xxhash3-64bit) of files in selected directories.
    Checksum(checksum::Args),
    /// Removes directories that do not contain any files.
    CleanEmptyDirectory(clean_empty_directory::Args),
    /// List files with similar file names.
    CleanFilename(clean_filename::Args),
    /// Finds duplicate files and conditionally deletes them.
    Duplicate(duplicate::Args),
    /// Delete files from file lists.
    Rmlist(rmlist::Args),
    /// List files with similar file names.
    SimilarName(similar_name::Args),
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Canonical(args) => canonical::run(args, cli.verbose),
        Commands::Checksum(args) => checksum::run(args, cli.verbose),
        Commands::CleanEmptyDirectory(args) => clean_empty_directory::run(args, cli.verbose),
        Commands::CleanFilename(args) => clean_filename::run(args, cli.verbose),
        Commands::Duplicate(args) => duplicate::run(args, cli.verbose),
        Commands::Rmlist(args) => rmlist::run(args, cli.verbose),
        Commands::SimilarName(args) => similar_name::run(args, cli.verbose),
    }
}
