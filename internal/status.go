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

package internal

import (
	"github.com/gosuri/uilive"
)

type status struct {
	w *uilive.Writer
}

func (s *status) Close() {
	s.w.Stop()
}

func newStatus() *status {
	w := uilive.New()
	w.Start()
	return &status{
		w: w,
	}
}

// Walking File System: 50 files, 900KiB
// Hashing Files: 10/50 files, 100KiB/900KiB
