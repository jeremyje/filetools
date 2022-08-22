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

package localfs

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/jeremyje/filetools/testdata"
)

func TestFileExists(t *testing.T) {
	var testCases = []struct {
		path string
		want bool
	}{
		{"does-not-exist", false},
		{"../testdata/hasdupes/a.1", true},
		{"../testdata/hasdupes/", false},
		{".", false},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(tc.path, func(t *testing.T) {
			t.Parallel()

			fullPath := testdata.Get(t, tc.path)
			got := FileExists(fullPath)
			if diff := cmp.Diff(tc.want, got); diff != "" {
				t.Errorf("Get mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

func TestDirExists(t *testing.T) {
	var testCases = []struct {
		path string
		want bool
	}{
		{"does-not-exist", false},
		{"../testdata/hasdupes/a.1", false},
		{"../testdata/hasdupes/", true},
		{".", true},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(tc.path, func(t *testing.T) {
			t.Parallel()

			fullPath := testdata.Get(t, tc.path)
			got := DirExists(fullPath)
			if diff := cmp.Diff(tc.want, got); diff != "" {
				t.Errorf("Get mismatch (-want +got):\n%s", diff)
			}
		})
	}
}
