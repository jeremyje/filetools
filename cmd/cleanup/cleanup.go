// Copyright 2020 Jeremy Edwards
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

// Package main cleans up directories.
package main

import (
	"flag"

	_ "net/http/pprof"

	"github.com/jeremyje/filetools/internal"
)

var (
	pathFlag   = flag.String("path", "", "Comma separated list of paths to scan.")
	dryRunFlag = flag.Bool("dry_run", true, "Reports but actions that would be taken but does not actually do them.")
)

func main() {
	internal.Initialize()
	internal.Check(internal.Cleanup(fromFlags()))
}

func fromFlags() *internal.CleanupParams {
	return &internal.CleanupParams{
		Paths:  internal.StringList(pathFlag),
		DryRun: *dryRunFlag,
	}
}
