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

// Package main deletes files from a file list.
package main

import (
	"flag"

	"github.com/jeremyje/filetools/internal"
	"github.com/jeremyje/filetools/internal/rmlist"
)

var (
	pathFlag     = flag.String("path", "", "File containing files to delete.")
	dryRunFlag   = flag.Bool("dry_run", true, "Dry run (print only) actions.")
	allowDirFlag = flag.Bool("allow_dir", false, "Allow directories to be deleted.")
)

func main() {
	internal.Initialize()
	rmlist.Run(fromFlags())
}

func fromFlags() *rmlist.Params {
	return &rmlist.Params{
		Paths:    internal.StringList(pathFlag),
		DryRun:   *dryRunFlag,
		AllowDir: *allowDirFlag,
	}
}
