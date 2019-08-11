package main

import (
	"flag"
	"github.com/jeremyje/filetools/internal"
)

var (
	pathFlag      = flag.String("path", "", "Comma separated list of paths to scan.")
	minSizeFlag   = flag.Int64("min_size", 0, "Minimize size of file to scan (in bytes).")
	deleteFlag    = flag.String("delete", "", "Comma separated list of file patterns that can be deleted if they are duplicates.")
	dryRunFlag    = flag.Bool("dry_run", true, "Reports but actions that would be taken (like deleting duplicates) but does not actually do them.")
	outputFlag    = flag.String("output", "", "Output file path for all duplicate files that were found.")
	verboseFlag   = flag.Bool("verbose", false, "Log extended information about the unique file scan.")
	overwriteFlag = flag.Bool("overwrite", true, "Overwrite output file if it already exists.")
	hashFlag      = flag.String("hash", "crc64", "Hash algorithm to use to compare similar files.")
)

func main() {
	flag.Parse()
	internal.Check(internal.Unique(fromFlags()))
}

func fromFlags() *internal.UniqueParams {
	return &internal.UniqueParams{
		Paths:        internal.StringList(pathFlag),
		MinSize:      *minSizeFlag,
		DeletePaths:  internal.StringList(deleteFlag),
		DryRun:       *dryRunFlag,
		ReportFile:   *outputFlag,
		Verbose:      *verboseFlag,
		Overwrite:    *overwriteFlag,
		HashFunction: *hashFlag,
	}
}
