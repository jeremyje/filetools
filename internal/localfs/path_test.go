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
	"fmt"
	"os"
	"path/filepath"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestDirList(t *testing.T) {
	wd, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}

	var testCases = []struct {
		input []string
		want  []string
	}{
		{
			input: []string{},
			want:  []string{},
		},
		{
			input: []string{".", "/", "/tmp"},
			want:  []string{"/"},
		},
		{
			input: []string{wd, ".", filepath.Join(wd, "ok")},
			want:  []string{wd},
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("%v", tc.input), func(t *testing.T) {
			t.Parallel()

			got, err := DirList(tc.input)
			if err != nil {
				t.Error(err)
			} else {
				if diff := cmp.Diff(tc.want, got); diff != "" {
					t.Errorf("Get mismatch (-want +got):\n%s", diff)
				}
			}
		})
	}
}

func TestAbsPaths(t *testing.T) {
	wd, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}

	var testCases = []struct {
		input []string
		want  []string
	}{
		{
			input: []string{"."},
			want:  []string{wd},
		},
		{
			input: []string{".", "/", "/tmp"},
			want:  []string{wd, "/", "/tmp"},
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("%v", tc.input), func(t *testing.T) {
			t.Parallel()

			got, err := absPaths(tc.input)
			if err != nil {
				t.Error(err)
			} else {
				if diff := cmp.Diff(tc.want, got); diff != "" {
					t.Errorf("Get mismatch (-want +got):\n%s", diff)
				}
			}
		})
	}
}

func TestSimplifyPaths(t *testing.T) {
	var testCases = []struct {
		input []string
		want  []string
	}{
		{
			input: []string{},
			want:  []string{},
		},
		{
			input: []string{"."},
			want:  []string{"."},
		},
		{
			input: []string{"/abc", "/def", "/"},
			want:  []string{"/"},
		},
		{
			input: []string{".", "/", "/tmp"},
			want:  []string{".", "/"},
		},
		{
			input: []string{"/a", "/abc", "/def", "/tmp"},
			want:  []string{"/a", "/abc", "/def", "/tmp"},
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("%v", tc.input), func(t *testing.T) {
			t.Parallel()

			got := simplifyPaths(tc.input)
			if diff := cmp.Diff(tc.want, got); diff != "" {
				t.Errorf("Get mismatch (-want +got):\n%s", diff)
			}
		})
	}
}
