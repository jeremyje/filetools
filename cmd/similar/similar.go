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

// Package main finds similarly named files in a directory structure.
package main

import (
	"flag"
	"github.com/jeremyje/filetools/internal"
)

var (
	pathFlag       = flag.String("path", "", "Comma separated list of directory paths to scan for similar files.")
	clearTokenFlag = flag.String("clear", "", "Clear tokens")
)

func main() {
	flag.Parse()
	internal.Similar(fromFlags())
}

func fromFlags() *internal.SimilarParams {
	return &internal.SimilarParams{
		Paths:       internal.StringList(pathFlag),
		ClearTokens: internal.StringList(clearTokenFlag),
	}
}
