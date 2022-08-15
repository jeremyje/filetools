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

package rmlist

import (
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestRmlist(t *testing.T) {
	var testCases = []struct {
		name     string
		start    []string
		targets  []string
		want     []string
		dryRun   bool
		allowDir bool
	}{
		{
			name:     "empty",
			start:    []string{},
			targets:  []string{},
			want:     []string{},
			dryRun:   false,
			allowDir: false,
		},
		{
			name:     "odds",
			start:    []string{"1", "2", "3", "4", "5"},
			targets:  []string{"1", "3", "5"},
			want:     []string{"2", "4"},
			dryRun:   false,
			allowDir: false,
		},
		{
			name:     "odds dry run",
			start:    []string{"1", "2", "3", "4", "5"},
			targets:  []string{"1", "3", "5"},
			want:     []string{"1", "2", "3", "4", "5"},
			dryRun:   true,
			allowDir: false,
		},
		{
			name:     "evens in dir",
			start:    []string{"a/1", "a/2", "a/3", "b/1", "b/2"},
			targets:  []string{"1", "3", "a/2", "b/2"},
			want:     []string{"a", "a/1", "a/3", "b", "b/1"},
			dryRun:   false,
			allowDir: false,
		},
		{
			name:     "evens in dir dry run",
			start:    []string{"a/1", "a/2", "a/3", "b/1", "b/2"},
			targets:  []string{"1", "3", "a/2", "b/2"},
			want:     []string{"a", "a/1", "a/2", "a/3", "b", "b/1", "b/2"},
			dryRun:   true,
			allowDir: false,
		},
		{
			name:     "delete a/ not allowed",
			start:    []string{"a/1", "a/2", "a/3", "b/1", "b/2"},
			targets:  []string{"a/"},
			want:     []string{"a", "a/1", "a/2", "a/3", "b", "b/1", "b/2"},
			dryRun:   false,
			allowDir: false,
		},
		{
			name:     "delete a/ allowed",
			start:    []string{"a/1", "a/2", "a/3", "b/1", "b/2"},
			targets:  []string{"a/"},
			want:     []string{"b", "b/1", "b/2"},
			dryRun:   false,
			allowDir: true,
		},
		{
			name:     "delete a/ allowed dry run",
			start:    []string{"a/1", "a/2", "a/3", "b/1", "b/2"},
			targets:  []string{"a/"},
			want:     []string{"a", "a/1", "a/2", "a/3", "b", "b/1", "b/2"},
			dryRun:   true,
			allowDir: true,
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			dir, err := os.MkdirTemp(os.TempDir(), "TestRmlist")
			if err != nil {
				t.Fatal(err)
			}
			t.Cleanup(func() {
				os.RemoveAll(dir)
			})
			rmFile, err := os.CreateTemp("", "TestRmlist")
			if err != nil {
				t.Fatal(err)
			}
			t.Cleanup(func() {
				os.Remove(rmFile.Name())
			})
			for _, target := range tc.targets {
				rmFile.WriteString(filepath.Join(dir, target) + "\n")
			}
			if err := rmFile.Close(); err != nil {
				t.Fatal(err)
			}

			for _, base := range tc.start {
				fullPath := filepath.Join(dir, base)
				baseDir := filepath.Dir(fullPath)
				if err := os.MkdirAll(baseDir, 0750); err != nil {
					t.Fatal(err)
				}
				if err := os.WriteFile(fullPath, []byte(fullPath), 0750); err != nil {
					t.Fatal(err)
				}
			}

			m, err := rmlist(&Params{
				Paths:    []string{rmFile.Name()},
				DryRun:   tc.dryRun,
				AllowDir: tc.allowDir,
			})
			report(m, err)
			if err != nil {
				t.Error(err)
			}
			wantM := map[string]any{}
			for _, v := range tc.want {
				wantM[v] = nil
			}
			for k, delErr := range m {
				if _, ok := wantM[k]; ok {
					if strings.Contains(delErr.Error(), "-dry_run") {
						t.Errorf("attempted to delete wanted file '%s'", k)
					}
				}
			}

			remainingFiles := listFilesInDir(t, dir)
			if diff := cmp.Diff(tc.want, remainingFiles); diff != "" {
				t.Errorf("rmlist() mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

func listFilesInDir(t *testing.T, path string) []string {
	files := []string{}

	filepath.WalkDir(path, func(walkedFilePath string, d os.DirEntry, err error) error {
		relPath, relErr := filepath.Rel(path, walkedFilePath)
		if relErr != nil {
			t.Error(relErr)
		} else {
			if relPath != "." {
				files = append(files, relPath)
			}
		}
		return nil
	})
	sort.Strings(files)
	return files
}
