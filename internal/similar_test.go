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
	"fmt"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/jeremyje/filetools/internal/localfs"
	"github.com/jeremyje/filetools/testdata"
)

func TestNormalize(t *testing.T) {
	var tc = []struct {
		in         string
		out        string
		clearToken []string
	}{
		{"/Video/1.mp4", "1", []string{}},
		{"/Video/1-ok.mp4", "1ok", []string{"-"}},
		{"/Video/1-ok.mp4", "1", []string{"-ok"}},
		{"/Video/1 0 1.mp4", "101", []string{""}},
		{"/Video/101.mp4", "101", []string{}},
		{"../testdata/similar/close/house.txt", "ouse", []string{"h", "m"}},
		{"../testdata/similar/close/mouse.pdf", "ouse", []string{"h", "m"}},
	}
	for _, tt := range tc {
		p := normalize(tt.in, tt.clearToken)
		if p != tt.out {
			t.Errorf("normalize(%s) > %s, expected %s", tt.in, p, tt.out)
		}
	}
}

func TestFindSimilarFiles(t *testing.T) {
	var testCases = []struct {
		name    string
		params  *SimilarParams
		matches map[string][]string
	}{
		{
			"Nothing", &SimilarParams{
				Paths:       []string{},
				ClearTokens: []string{},
			},
			map[string][]string{},
		},
		{
			"Empty Path", &SimilarParams{
				Paths:       []string{""},
				ClearTokens: []string{},
			},
			map[string][]string{},
		},
		{
			"A, B Match", &SimilarParams{
				Paths:       []string{testdata.Get(t, "similar/by_extension")},
				ClearTokens: []string{""},
			},
			map[string][]string{
				"a": {testdata.Get(t, "similar/by_extension/a.1"), testdata.Get(t, "similar/by_extension/a.2")},
				"b": {testdata.Get(t, "similar/by_extension/b.1"), testdata.Get(t, "similar/by_extension/b.2")},
			},
		},
		{
			"A, B Match (all directories)", &SimilarParams{
				Paths:       []string{testdata.Get(t, "similar")},
				ClearTokens: []string{""},
			},
			map[string][]string{
				"a": {testdata.Get(t, "similar/by_extension/a.1"), testdata.Get(t, "similar/by_extension/a.2")},
				"b": {testdata.Get(t, "similar/by_extension/b.1"), testdata.Get(t, "similar/by_extension/b.2")},
			},
		},
		{
			"Similar Files no match wrong tokens", &SimilarParams{
				Paths:       []string{testdata.Get(t, "similar/close")},
				ClearTokens: []string{"txt", "pdf"},
			},
			map[string][]string{},
		},
		{
			"Similar Files Match by tokens", &SimilarParams{
				Paths:       []string{testdata.Get(t, "similar/close")},
				ClearTokens: []string{"m", "h"},
			},
			map[string][]string{
				"ouse": {testdata.Get(t, "similar/close/house.txt"), testdata.Get(t, "similar/close/mouse.pdf")},
			},
		},
		{
			"Similar Match (all directories)", &SimilarParams{
				Paths:       []string{testdata.Get(t, "similar")},
				ClearTokens: []string{"m", "h"},
			},
			map[string][]string{
				"a":    {testdata.Get(t, "similar/by_extension/a.1"), testdata.Get(t, "similar/by_extension/a.2")},
				"b":    {testdata.Get(t, "similar/by_extension/b.1"), testdata.Get(t, "similar/by_extension/b.2")},
				"ouse": {testdata.Get(t, "similar/close/house.txt"), testdata.Get(t, "similar/close/mouse.pdf")},
			},
		},
		{
			"Tokens Don't Match Anything", &SimilarParams{
				Paths:       []string{testdata.Get(t, "similar/notsimilar")},
				ClearTokens: []string{"$", "#", "one"},
			},
			map[string][]string{},
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("%s findSimilarFiles(%+v)", tc.name, tc.params), func(t *testing.T) {
			similarMap := findSimilarFiles(tc.params)
			if len(tc.matches) == 0 {
				if len(similarMap) != 0 {
					t.Errorf("want %+v to be empty", similarMap)
				}
			} else {
				want := localfs.FromSlashMap(tc.matches)
				got := localfs.FromSlashMap(similarMap)
				if diff := cmp.Diff(want, got); diff != "" {
					t.Errorf("Get mismatch (-want +got):\n%s", diff)
				}
			}
		})
	}
}
