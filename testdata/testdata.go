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

// Package testdata provides access to testdata directory programmatically from go.
package testdata

import (
	"os"
	"path/filepath"
	"testing"
)

const (
	canonicalFile = "testdata/examples/file_example_favicon.ico"
)

// Get the test file path.
func Get(tb testing.TB, path string) string {
	return filepath.Join(GetDirectory(tb), path)
}

// GetDirectory returns the fully qualified local directory of where the testdata files are located.
func GetDirectory(tb testing.TB) string {
	wdDir, err := os.Getwd()
	if err != nil {
		tb.Fatalf("cannot get current working directory: %s", err)
	}

	dir, err := filepath.Abs(wdDir)
	if err != nil {
		tb.Fatalf("cannot get absolute current working directory: %s", err)
	}

	for {
		exampleFile := filepath.Join(dir, canonicalFile)
		if _, err := os.Stat(exampleFile); err == nil {
			break
		}
		nextDir := filepath.Dir(dir)
		if nextDir == dir || len(nextDir) < 3 {
			tb.Fatalf("cannot find testdata/examples/ directory within working directory '%s'. was this test run outside of the project?", wdDir)
		}
		dir = nextDir
	}

	return filepath.Join(dir, "testdata")
}
