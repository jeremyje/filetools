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
	"path/filepath"
	"sort"
)

// DirList returns a sorted, de-duped, absolute path list of the local directory paths entered.
func DirList(dirs []string) ([]string, error) {
	absDirs, err := absPaths(dirs)
	if err != nil {
		return nil, err
	}
	return simplifyPaths(absDirs), nil
}

func sortUniquePaths(paths []string) []string {
	m := map[string]any{}

	for _, path := range paths {
		m[path] = nil
	}

	v := []string{}
	for k := range m {
		v = append(v, k)
	}
	sort.Strings(v)
	return v
}

func simplifyPaths(paths []string) []string {
	if len(paths) == 0 {
		return []string{}
	}

	sortedPaths := sortUniquePaths(paths)
	v := []string{sortedPaths[0]}
	prev := sortedPaths[0]
	for _, p := range sortedPaths {

		match := false
		subPaths := explodePath(p)
		for _, sp := range subPaths {
			if sp == prev {
				match = true
				break
			}
		}
		if !match {
			prev = p
			v = append(v, p)
		}
	}
	return v
}

func explodePath(path string) []string {
	dirs := []string{path}

	parentPath := filepath.Dir(path)
	for parentPath != path {
		path = parentPath
		dirs = append(dirs, path)
		parentPath = filepath.Dir(path)
	}
	return dirs
}

func absPaths(paths []string) ([]string, error) {
	absPaths := []string{}
	for _, path := range paths {
		absPath, err := filepath.Abs(path)
		if err != nil {
			return []string{}, err
		}
		absPaths = append(absPaths, absPath)
	}
	return absPaths, nil
}
