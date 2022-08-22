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
	"io"
	"log"
	"os"
	"strings"
	"sync"
	"time"

	//"github.com/gosuri/uilive"
	"github.com/pkg/errors"
)

const (
	coarseHashMinFileSize = 10 * 1024 * 1024
	coarseHashChunkSize   = 64 * 1024
)

// UniqueParams is the parameters for finding duplicate files in a directory structure.
type UniqueParams struct {
	Paths           []string
	MinSize         int64
	DeletePaths     []string
	DryRun          bool
	ReportFile      string
	Verbose         bool
	Overwrite       bool
	HashFunction    string
	StatusFrequency time.Duration
	CoarseHashing   bool
	EnableFilePprof bool
}

type uniqueScanMetrics struct {
	fileSizeCounter           *counter
	fileCounter               *counter
	filesToHash               *counter
	filesToHashBytes          *counter
	processedFilesToHash      *counter
	processedFilesToHashBytes *counter
}

func (usm *uniqueScanMetrics) print() {
	fmt.Printf("metrics\n\n")
	fmt.Printf("%s\n", usm.fileCounter.String())
	fmt.Printf("%s\n", usm.fileSizeCounter.String())
	fmt.Printf("%s\n", usm.filesToHash.String())
	fmt.Printf("%s\n", usm.filesToHashBytes.String())
	fmt.Printf("%s\n", usm.processedFilesToHash.String())
	fmt.Printf("%s\n", usm.processedFilesToHashBytes.String())
	fmt.Printf("\n\n")
}

func newUniqueScanMetrics() *uniqueScanMetrics {
	return &uniqueScanMetrics{
		fileSizeCounter:           newCounter("file-size"),
		fileCounter:               newCounter("files"),
		filesToHash:               newCounter("files-to-hash"),
		filesToHashBytes:          newCounter("files-to-hash-by-bytes"),
		processedFilesToHash:      newCounter("processed-files-to-hash"),
		processedFilesToHashBytes: newCounter("processed-files-to-hash-by-bytes"),
	}
}

type uniqueStatus struct {
	label string
	rootM *measure
	//w               *uilive.Writer
	currentM        *measure
	lastWrite       time.Time
	updateFrequency time.Duration
	enableFilePprof bool
	startTime       time.Time
}

func newUniqueStatus(updateFrequency time.Duration, enableFilePprof bool) *uniqueStatus {
	//w := uilive.New()
	//w.RefreshInterval = updateFrequency
	//w.Start()
	return &uniqueStatus{
		//	w:               w,
		label:           "Starting Scan...",
		rootM:           newMeasure("Duplicate File Scan"),
		updateFrequency: updateFrequency,
		enableFilePprof: enableFilePprof,
		startTime:       time.Now(),
	}
}

// Close stops live update status line.
func (us *uniqueStatus) Close() {
	us.rootM.done()
	//us.w.Stop()
}

func (us *uniqueStatus) set(label string) {
	us.label = label
	if us.currentM != nil {
		us.currentM.done()
	}
	us.currentM = us.rootM.sub(label)
	//fmt.Fprintf(us.w, "%s\n", label)
	log.Println(label)
	if us.enableFilePprof {
		stopCPUProfile()
		startCPUProfile(us.startTime.Format("20060102150405") + strings.ReplaceAll(label, " ", "") + ".pprof")
	}
}

func (us *uniqueStatus) detail(d string) {
	now := time.Now()
	if us.lastWrite.Add(us.updateFrequency).Before(now) {
		log.Printf("%s: %s\n\n", us.label, d)
		//fmt.Fprintf(us.w, "%s: %s\n\n", us.label, d)
		us.lastWrite = now
	}
}

type fileData struct {
	name       string
	coarseHash string
	hash       string
	hashError  error
	m          sync.RWMutex
}

func (fd *fileData) getCoarseHash() (string, error) {
	if len(fd.coarseHash) > 0 {
		return fd.coarseHash, nil
	}
	if fd.hashError != nil {
		return "", fd.hashError
	}
	coarseHash, err := coarseHashFile(fd.name, "crc64", coarseHashChunkSize)
	fd.hashError = err
	fd.coarseHash = coarseHash
	return coarseHash, err
}

func (fd *fileData) getHash(hashFunction string) (string, error) {
	fd.m.RLock()
	hash := fd.hash
	hashError := fd.hashError
	name := fd.name
	fd.m.RUnlock()

	if len(hash) > 0 {
		return hash, nil
	}
	if hashError != nil {
		return "", hashError
	}

	fileHash, err := hashFile(name, hashFunction)
	fd.m.Lock()
	fd.hashError = err
	fd.hash = fileHash
	fd.m.Unlock()
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
	filesBySize  map[int64]*filesWithSameSize
	metrics      *uniqueScanMetrics
	status       *uniqueStatus
	hashFunction string
}

func newUniqueWalkShard(status *uniqueStatus, metrics *uniqueScanMetrics, hashFunction string) *uniqueWalkShard {
	return &uniqueWalkShard{
		filesBySize:  map[int64]*filesWithSameSize{},
		metrics:      metrics,
		status:       status,
		hashFunction: hashFunction,
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
	us.metrics.fileSizeCounter.incBy(size)
	us.metrics.fileCounter.inc()
	data := &fileData{
		name: path,
	}
	if val, ok := us.filesBySize[size]; ok {
		val.append(data)
	} else {
		us.filesBySize[size] = newFilesWithSameSize(data)
	}
	us.status.detail(fmt.Sprintf("%d files, %s", us.metrics.fileCounter.value(), sizeString(us.metrics.fileSizeCounter.value())))
	return nil
}

func (us *uniqueWalkShard) hashFiles(enableCoarseHashing bool) error {
	for size, f := range us.filesBySize {
		numFiles := len(f.files)
		if numFiles > 1 {
			us.metrics.filesToHash.incBy(int64(numFiles))
			us.metrics.filesToHashBytes.incBy(int64(numFiles) * size)
		} else {
			delete(us.filesBySize, size)
		}
	}
	us.metrics.print()

	if enableCoarseHashing {
		fbsLen := len(us.filesBySize)
		i := 0
		for size, f := range us.filesBySize {
			us.status.detail(fmt.Sprintf("Coarse Hash: %d of %d", i, fbsLen))
			i++
			if len(f.files) > 1 {
				if size > coarseHashMinFileSize {
					matches := map[string][]*fileData{}
					newFiles := []*fileData{}
					for _, fd := range f.files {
						hash, err := fd.getCoarseHash()
						if err != nil {
							log.Printf("error coarse hashing %s, skipping\n", fd.name)
							newFiles = append(newFiles, fd)
							continue
						}
						v, ok := matches[hash]
						if ok {
							matches[hash] = append(v, fd)
						} else {
							matches[hash] = []*fileData{fd}
						}
					}

					for _, v := range matches {
						if len(v) > 1 {
							newFiles = append(newFiles, v...)
						}
					}
					f.files = newFiles
				}
			}
		}
		us.metrics.print()
	}

	hashFunction := us.hashFunction
	numFiles := int64(0)
	totalSize := int64(0)
	for size, fdList := range us.filesBySize {
		n := int64(len(fdList.files))
		numFiles += n
		totalSize += n * size
	}
	us.metrics.filesToHash.set(numFiles)
	us.metrics.filesToHashBytes.set(totalSize)
	i := 0
	for size, f := range us.filesBySize {
		us.status.detail(fmt.Sprintf("%s Hash: %d of %d", us.hashFunction, i, numFiles))
		i++
		if len(f.files) > 1 {
			matches := map[string][]*fileData{}
			newFiles := []*fileData{}
			var wg sync.WaitGroup

			for _, fd := range f.files {
				targetFd := fd
				wg.Add(1)
				go func() {
					targetFd.getHash(hashFunction)
					wg.Done()
				}()
			}
			wg.Wait()
			for _, fd := range f.files {
				hash, err := fd.getHash(us.hashFunction)
				if err != nil {
					newFiles = append(newFiles, fd)
					continue
				}
				v, ok := matches[hash]
				if ok {
					matches[hash] = append(v, fd)
				} else {
					matches[hash] = []*fileData{fd}
				}
				us.metrics.processedFilesToHash.inc()
				us.metrics.processedFilesToHashBytes.incBy(size)
				us.status.detail(fmt.Sprintf("%d/%d files, %s/%s",
					us.metrics.processedFilesToHash.value(), us.metrics.filesToHash.value(),
					sizeString(us.metrics.processedFilesToHashBytes.value()), sizeString(us.metrics.filesToHashBytes.value())))
				if err != nil {
					return err
				}
			}

			for _, v := range matches {
				if len(v) > 1 {
					newFiles = append(newFiles, v...)
				}
			}
			f.files = newFiles
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
				hash, _ := fileData.getHash(us.hashFunction)
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
	shards       []*uniqueWalkShard
	metrics      *uniqueScanMetrics
	status       *uniqueStatus
	hashFunction string
}

func (uc *uniqueContext) NewWalkShard() func(string, os.FileInfo, error) error {
	s := newUniqueWalkShard(uc.status, uc.metrics, uc.hashFunction)
	uc.shards = append(uc.shards, s)
	return s.accept
}

func (uc *uniqueContext) dump() {
	for _, shard := range uc.shards {
		for size, files := range shard.filesBySize {
			log.Printf("%d %s\n", size, files.String())
		}
	}
}

func (uc *uniqueContext) hashFiles(enableCoarseHashing bool) error {
	if len(uc.shards) > 1 {
		return errors.New("cannot has multi-sharded context, use mergeFrom")
	}
	if len(uc.shards) == 1 {
		return uc.shards[0].hashFiles(enableCoarseHashing)
	}
	return nil
}

func newUniqueContext(updateFrequency time.Duration, enableFilePprof bool, hashFunction string) *uniqueContext {
	return &uniqueContext{
		metrics:      newUniqueScanMetrics(),
		status:       newUniqueStatus(updateFrequency, enableFilePprof),
		hashFunction: hashFunction,
	}
}

func (uc *uniqueContext) merge() *uniqueContext {
	if len(uc.shards) > 1 {
		merged := &uniqueContext{
			metrics: uc.metrics,
			status:  uc.status,
		}
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

// Close closes the context.
func (uc *uniqueContext) Close() {
	uc.status.Close()
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
	uc := newUniqueContext(p.StatusFrequency, p.EnableFilePprof, p.HashFunction)
	defer uc.Close()

	uc.status.set("Scan Files")
	err := shardedMultiwalk(p.Paths, uc)
	if err != nil {
		return nil, errors.Wrap(err, "cannot scan files for uniqueness")
	}
	uc.metrics.print()

	uc.status.set("Merge Scans")
	uc = uc.merge()

	uc.status.set("Hash Candidates")
	uc.hashFiles(p.CoarseHashing)
	if p.Verbose {
		uc.dump()
	}

	uc.status.set("Group Duplicates")
	report := uc.findDuplicates()
	if p.Verbose {
		for hash, dup := range report.Duplicates {
			fmt.Printf("%s %+v\n", hash, dup.Names)
		}
	}

	uc.status.set("Render Report")
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
