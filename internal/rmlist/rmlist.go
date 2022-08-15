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
	"bufio"
	"fmt"
	"log"
	"os"
	"sort"
)

// Params are the parameters for deleting a list of files.
type Params struct {
	Paths    []string
	DryRun   bool
	AllowDir bool
}

// Run rmlist application.
func Run(p *Params) {
	results, err := rmlist(p)
	report(results, err)
}

func getPathsFromFiles(deleteListFiles []string) ([]string, error) {
	var err error
	m := map[string]any{}
	for _, deleteListFile := range deleteListFiles {
		var f *os.File
		f, err = os.Open(deleteListFile)
		if err != nil {
			log.Printf("cannot read delete list file '%s', %s", deleteListFile, err)
		}
		defer f.Close()
		s := bufio.NewScanner(f)

		s.Split(bufio.ScanLines)

		for s.Scan() {
			m[s.Text()] = nil
		}
	}
	targets := []string{}
	for k := range m {
		targets = append(targets, k)
	}
	sort.Strings(targets)
	if len(targets) == 0 {
		return targets, err
	}
	return targets, nil
}

func rmlist(p *Params) (map[string]error, error) {
	m := map[string]error{}
	deleteTargets, err := getPathsFromFiles(p.Paths)
	if err != nil {
		return m, err
	}
	for _, deleteTarget := range deleteTargets {
		deleteFromFilesystem(m, deleteTarget, p.DryRun, p.AllowDir)
	}

	return m, nil
}

func deleteFromFilesystem(m map[string]error, path string, dryRun bool, allowDir bool) {
	info, err := os.Stat(path)
	if err != nil {
		m[path] = nil
		return
	}
	if info.Mode()&os.ModeSymlink == os.ModeSymlink {
		m[path] = fmt.Errorf("%s is a symbolic link, not supported yet", path)
		return
	}
	if info.IsDir() {
		if allowDir {
			if dryRun {
				m[path] = fmt.Errorf("%s was NOT deleted because -dry_run=true", path)
			} else {
				m[path] = os.RemoveAll(path)
			}
		} else {
			m[path] = fmt.Errorf("%s is a directory, use -allow_dir=true to enable deletion of directories", path)
		}
		return
	}
	if dryRun {
		m[path] = fmt.Errorf("%s was NOT deleted because -dry_run=true", path)
	} else {
		m[path] = os.Remove(path)
	}
}

func report(targets map[string]error, err error) {
	if err != nil {
		log.Printf("Failed to delete file set: %s", err)
	}
	log.Println("Deleted Files")

	t := []string{}
	for k := range targets {
		t = append(t, k)
	}
	for _, k := range t {
		err := targets[k]
		if err == nil {
			log.Printf("[DELETED] %s", k)
		} else {
			log.Printf("[FAILED ] %s -- %s", k, err)
		}
	}
}
