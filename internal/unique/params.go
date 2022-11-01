// Copyright 2022 Jeremy Edwards
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

package unique

import (
	"time"
)

const (
	coarseHashMinFileSize = 10 * 1024 * 1024
	coarseHashChunkSize   = 64 * 1024
)

// Params for finding duplicate files in a directory structure.
type Params struct {
	Paths           []string
	MinSize         int64
	DeletePaths     []string
	DryRun          bool
	ReportFile      string
	Verbose         bool
	Overwrite       bool
	HashFunction    string
	StatusFrequency time.Duration
	CoarseHashing   bool
	EnableFilePprof bool
}
