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

package internal

import (
	"log"
	"os"
	"path/filepath"

	"github.com/jeremyje/filetools/internal/localfs"
	"github.com/pkg/errors"
)

// CleanupParams is the parameters for cleaning up a directory.
type CleanupParams struct {
	Paths  []string
	DryRun bool
}

// Cleanup cleans up a directory structure.
func Cleanup(params *CleanupParams) error {
	var lastErr error
	paths, err := localfs.DirList(params.Paths)
	if err != nil {
		return err
	}
	for _, path := range paths {
		if _, err = purgeEmptyDirectory(path, params.DryRun); err != nil {
			log.Printf("error scanning '%s', %s", path, err)
			lastErr = err
		}
	}
	return lastErr
}

func purgeEmptyDirectory(path string, dryRun bool) (bool, error) {
	var lastErr error
	items, err := os.ReadDir(path)
	if err != nil {
		return false, errors.Wrapf(err, "cannot read '%s'", path)
	}
	hasFiles := false
	for _, item := range items {
		if item.IsDir() {
			var subDirHasFiles bool
			subDirHasFiles, lastErr = purgeEmptyDirectory(filepath.Join(path, item.Name()), dryRun)
			hasFiles = hasFiles || subDirHasFiles
		} else {
			hasFiles = true
		}
	}
	if !hasFiles {
		if dryRun {
			log.Printf("[DRY RUN] %s", path)
		} else {
			if err = os.Remove(path); err != nil {
				return hasFiles, errors.Wrapf(err, "cannot remove directory '%s'", path)
			}
			log.Printf("[DELETED] %s", path)
		}
	}
	return hasFiles, lastErr
}
