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
	"github.com/jeremyje/filetools/internal/treemap"
)

var (
	pathFlag    = flag.String("path", "", "File containing files to delete.")
	minSizeFlag = flag.Int64("min_size", 0, "Minimum size of file to map out.")
	outputFlag  = flag.String("output", "treemap.html", "Output path of the report.")
)

func main() {
	internal.Initialize()
	treemap.Run(fromFlags())
}

func fromFlags() *treemap.Params {
	return &treemap.Params{
		Paths:   internal.StringList(pathFlag),
		MinSize: *minSizeFlag,
		Output:  *outputFlag,
	}
}
