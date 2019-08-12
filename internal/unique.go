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
	"github.com/pkg/errors"
	"io"
	"os"
	"sync"
)

// UniqueParams is the parameters for finding duplicate files in a directory structure.
type UniqueParams struct {
	Paths        []string
	MinSize      int64
	DeletePaths  []string
	DryRun       bool
	ReportFile   string
	Verbose      bool
	Overwrite    bool
	HashFunction string
}

type sizeBucketedFiles struct {
	files map[int64]*sameSizeFileSet
}

type sameSizeFileSet struct {
	fileData []*fileData
}

type fileData struct {
	name      string
	hash      string
	hashError error
}

func (fd *fileData) getHash() (string, error) {
	if len(fd.hash) > 0 {
		return fd.hash, nil
	}
	if fd.hashError != nil {
		return "", fd.hashError
	}

	fileHash, err := hashFile(fd.name, "sha256")
	fd.hashError = err
	fd.hash = fileHash
	return fileHash, err
}

// FilesWithSameHash holds a list of files with the same hash code.
type FilesWithSameHash struct {
	Names []string
	Size  int64
}

// DuplicateFileReport is a report of all the duplicate files.
type DuplicateFileReport struct {
	Title      string
	Duplicates map[string]*FilesWithSameHash
}

type filesWithSameSize struct {
	m     sync.Mutex
	files []*fileData
}

func (fs *filesWithSameSize) mergeFrom(source *filesWithSameSize) {
	fs.m.Lock()
	fs.files = append(fs.files, source.files...)
	fs.m.Unlock()
}

func (fs *filesWithSameSize) append(data *fileData) {
	fs.m.Lock()
	fs.files = append(fs.files, data)
	fs.m.Unlock()
}

func (fs *filesWithSameSize) String() string {
	filenames := []string{}
	for _, file := range fs.files {
		filenames = append(filenames, file.name+"$"+file.hash)
	}
	return fmt.Sprintf("%+v", filenames)
}

func newFilesWithSameSize(first *fileData) *filesWithSameSize {
	return &filesWithSameSize{
		files: []*fileData{first},
	}
}

type uniqueWalkShard struct {
	filesBySize map[int64]*filesWithSameSize
}

func newUniqueWalkShard() *uniqueWalkShard {
	return &uniqueWalkShard{
		filesBySize: map[int64]*filesWithSameSize{},
	}
}

func (us *uniqueWalkShard) mergeFrom(source *uniqueWalkShard) {
	for size, fs := range source.filesBySize {
		if val, ok := us.filesBySize[size]; ok {
			val.mergeFrom(fs)
		} else {
			us.filesBySize[size] = fs
		}
	}
}

func (us *uniqueWalkShard) accept(path string, info os.FileInfo, err error) error {
	size := info.Size()
	data := &fileData{
		name: path,
	}
	if val, ok := us.filesBySize[size]; ok {
		val.append(data)
	} else {
		us.filesBySize[size] = newFilesWithSameSize(data)
	}
	return nil
}

func (us *uniqueWalkShard) hashFiles() error {
	for _, f := range us.filesBySize {
		if len(f.files) > 1 {
			for _, fileData := range f.files {
				_, err := fileData.getHash()
				if err != nil {
					return err
				}
			}
		}
	}
	return nil
}

func newFileWithSameHash(size int64, first string) *FilesWithSameHash {
	return &FilesWithSameHash{
		Names: []string{first},
		Size:  size,
	}
}

func (us *uniqueWalkShard) findDuplicates() *DuplicateFileReport {
	filesByHash := map[string]*FilesWithSameHash{}
	for size, f := range us.filesBySize {
		if len(f.files) > 1 {
			for _, fileData := range f.files {
				hash, _ := fileData.getHash()
				if len(hash) > 0 {
					if val, ok := filesByHash[hash]; ok {
						val.Names = append(val.Names, fileData.name)
					} else {
						filesByHash[hash] = newFileWithSameHash(size, fileData.name)
					}
				}
			}
		}
	}
	for hash, val := range filesByHash {
		if len(val.Names) <= 1 {
			delete(filesByHash, hash)
		}
	}
	return &DuplicateFileReport{
		Title:      "Duplicate Files",
		Duplicates: filesByHash,
	}
}

type uniqueContext struct {
	shards []*uniqueWalkShard
}

func (uc *uniqueContext) NewWalkShard() func(string, os.FileInfo, error) error {
	s := newUniqueWalkShard()
	uc.shards = append(uc.shards, s)
	return s.accept
}

func (uc *uniqueContext) dump() {
	for _, shard := range uc.shards {
		for size, files := range shard.filesBySize {
			fmt.Printf("%d %s\n", size, files.String())
		}
	}
}

func (uc *uniqueContext) hashFiles() error {
	if len(uc.shards) > 1 {
		return errors.New("cannot has multi-sharded context, use mergeFrom")
	}
	if len(uc.shards) == 1 {
		return uc.shards[0].hashFiles()
	}
	return nil
}

func newUniqueContext() *uniqueContext {
	return &uniqueContext{}
}

func (uc *uniqueContext) merge() *uniqueContext {
	if len(uc.shards) > 1 {
		merged := newUniqueContext()
		merged.NewWalkShard()
		shard := merged.shards[0]
		for _, source := range uc.shards {
			shard.mergeFrom(source)
		}
		return merged
	}
	return uc
}

func (uc *uniqueContext) findDuplicates() *DuplicateFileReport {
	if len(uc.shards) == 1 {
		return uc.shards[0].findDuplicates()
	}
	return nil
}

// Unique finds all the duplicate files in a directory structure based on the UniqueParams
func Unique(p *UniqueParams) error {
	_, err := uniqueScan(p)
	if err != nil {
		return err
	}
	return nil
}

func uniqueScan(p *UniqueParams) (*uniqueContext, error) {
	uc := newUniqueContext()
	fmt.Printf("- scanning\n")
	err := shardedMultiwalk(p.Paths, uc)
	if err != nil {
		return nil, errors.Wrap(err, "cannot scan files for uniqueness")
	}
	fmt.Printf("- merge results\n")
	uc = uc.merge()
	fmt.Printf("- hashing\n")
	uc.hashFiles()
	if p.Verbose {
		uc.dump()
	}
	fmt.Printf("- find duplicates\n")
	report := uc.findDuplicates()
	fmt.Printf("\nDuplicates found\n")
	if p.Verbose {
		for hash, dup := range report.Duplicates {
			fmt.Printf("%s %+v\n", hash, dup.Names)
		}
	}
	err = reportDeplicates(p, report)
	if err != nil {
		return nil, err
	}
	return uc, nil
}

func reportDeplicates(p *UniqueParams, report *DuplicateFileReport) error {
	var w io.Writer
	w = os.Stdout
	if len(p.ReportFile) > 0 {
		f, err := openFileForWrite(p.ReportFile, p.Overwrite)
		if err != nil {
			return err
		}
		defer f.Close()
		w = f
	}
	return writeReport(w, duplicateFileReportTemplate, report)
}
