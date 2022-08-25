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

package treemap

import (
	"fmt"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/jeremyje/filetools/testdata"
)

func TestTreemap(t *testing.T) {
	var testCases = []struct {
		paths         []string
		wantSize      int64
		wantFileCount int64
	}{
		{
			paths:    []string{},
			wantSize: 0,
		},
		{
			paths:         []string{testdata.Get(t, "hasdupes")},
			wantSize:      5,
			wantFileCount: 6,
		},
		{
			paths:         []string{testdata.Get(t, "hasdupes"), testdata.Get(t, "nodupes")},
			wantSize:      11,
			wantFileCount: 9,
		},
		{
			paths:         []string{testdata.Get(t, "")},
			wantSize:      6840531,
			wantFileCount: 35,
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("%+v", tc.paths), func(t *testing.T) {
			t.Parallel()

			n, err := treemap(&Params{
				Paths: tc.paths,
			})
			if err != nil {
				t.Fatal(err)
			}
			if diff := cmp.Diff(tc.wantSize, n.Size()); diff != "" {
				t.Errorf("Size() mismatch (-want +got):\n%s", diff)
			}
			if diff := cmp.Diff(tc.wantFileCount, n.FileCount()); diff != "" {
				t.Errorf("FileCount() mismatch (-want +got):\n%s", diff)
			}
		})
	}
}
