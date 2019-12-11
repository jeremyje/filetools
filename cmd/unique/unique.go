// Copyright 2019 Jeremy Edwards
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

// Package main finds duplicate files in a directory structure.
package main

import (
	"flag"
	"time"

	"github.com/jeremyje/filetools/internal"
)

var (
	pathFlag            = flag.String("path", "", "Comma separated list of paths to scan.")
	minSizeFlag         = flag.Int64("min_size", 0, "Minimize size of file to scan (in bytes).")
	deleteFlag          = flag.String("delete", "", "Comma separated list of file patterns that can be deleted if they are duplicates.")
	dryRunFlag          = flag.Bool("dry_run", true, "Reports but actions that would be taken (like deleting duplicates) but does not actually do them.")
	outputFlag          = flag.String("output", "", "Output file path for all duplicate files that were found.")
	verboseFlag         = flag.Bool("verbose", false, "Log extended information about the unique file scan.")
	overwriteFlag       = flag.Bool("overwrite", true, "Overwrite output file if it already exists.")
	hashFlag            = flag.String("hash", "crc64", "Hash algorithm to use to compare similar files.")
	coarseHashFlag      = flag.Bool("coarse_hash", true, "Enables a preliminary hashing method to quickly split up obviously different files.")
	statusFrequencyFlag = flag.Duration("status_frequency", time.Second*5, "Frequency for updating status")
)

func main() {
	flag.Parse()
	internal.Check(internal.Unique(fromFlags()))
}

func fromFlags() *internal.UniqueParams {
	return &internal.UniqueParams{
		Paths:           internal.StringList(pathFlag),
		MinSize:         *minSizeFlag,
		DeletePaths:     internal.StringList(deleteFlag),
		DryRun:          *dryRunFlag,
		ReportFile:      *outputFlag,
		Verbose:         *verboseFlag,
		Overwrite:       *overwriteFlag,
		HashFunction:    *hashFlag,
		CoarseHashing:   *coarseHashFlag,
		StatusFrequency: *statusFrequencyFlag,
	}
}
