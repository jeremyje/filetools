// Copyright 2022 Jeremy Edwards
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

package unique

import (
	"fmt"
	"os"
	"sort"
	"sync"

	"github.com/jeremyje/filetools/internal/core"
	"github.com/jeremyje/filetools/internal/localfs"
	pb "github.com/jeremyje/filetools/internal/unique/proto"
	"go.uber.org/zap"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
)

type uniqueFileScanShard struct {
	db map[string]*pb.FileSummary
	sync.RWMutex
}

func (u *uniqueFileScanShard) acceptFile(name string, info os.FileInfo, err error) error {
	if err != nil {
		return err
	}

	u.Lock()
	u.db[name] = &pb.FileSummary{
		Name: name,
		Size: info.Size(),
	}
	u.Unlock()

	return nil
}

func newUniqueFileScanShard() *uniqueFileScanShard {
	return &uniqueFileScanShard{
		db: map[string]*pb.FileSummary{},
	}
}

type uniqueFileScanner struct {
	scanShards []*uniqueFileScanShard
	db         map[string]*pb.FileSummary
	bySize     map[int64][]*pb.FileSummary
}

func (u *uniqueFileScanner) NewWalkShard() func(string, os.FileInfo, error) error {
	s := newUniqueFileScanShard()
	u.scanShards = append(u.scanShards, s)
	return s.acceptFile
}

func (u *uniqueFileScanner) mergeAndIndex() {
	scanShards := u.scanShards
	for _, scanShard := range scanShards {
		scanShard.RLock()
		defer scanShard.RUnlock()
		for name, file := range scanShard.db {
			// Do not add 0-byte files since they should never be deleted. They tend to be markers.
			if file.GetSize() <= 0 {
				continue
			}
			// Add record to the dictionary.
			u.db[name] = file

			// Append by size
			val, ok := u.bySize[file.GetSize()]
			if !ok {
				val = []*pb.FileSummary{file}
			} else {
				val = append(val, file)
			}
			u.bySize[file.GetSize()] = val
		}
	}

	// Remove all entries that are unique sizes.
	for size, files := range u.bySize {
		if len(files) == 1 {
			delete(u.bySize, size)
			delete(u.db, files[0].GetName())
		}
	}
}

func (u *uniqueFileScanner) report() *pb.DuplicateFileReport {
	r := &pb.DuplicateFileReport{
		Duplicates: []*pb.DuplicateFileReport_DuplicateFileSet{},
	}

	keys := make([]int64, 0, len(u.bySize))
	for k := range u.bySize {
		keys = append(keys, k)
	}
	sort.Slice(keys, func(i, j int) bool { return keys[i] < keys[j] })

	for _, size := range keys {
		duplicateSet := &pb.DuplicateFileReport_DuplicateFileSet{
			Size: size,
			File: []*pb.FileSummary{},
		}
		for _, file := range u.bySize[size] {
			duplicateSet.File = append(duplicateSet.File, MustClone(file))
		}
		r.Duplicates = append(r.Duplicates, duplicateSet)
	}
	return r
}

func MustClone[T protoreflect.ProtoMessage](msg T) T {
	np := proto.Clone(msg)
	cp, ok := np.(T)
	if ok {
		return cp
	}
	panic("cannot clone protobuf")
}

func newScanner() *uniqueFileScanner {
	return &uniqueFileScanner{
		scanShards: []*uniqueFileScanShard{},
		db:         map[string]*pb.FileSummary{},
		bySize:     map[int64][]*pb.FileSummary{},
	}
}

func Run(params *Params) error {
	logBase, err := core.Logger()
	if err != nil {
		return err
	}
	_, err = run(logBase, params)
	return err
}

func run(logBase *zap.Logger, params *Params) (*uniqueFileScanner, error) {
	log := logBase.With(
		zap.Strings("Paths", params.Paths),
		zap.Int64("MinSize", params.MinSize),
		zap.Strings("DeletePaths", params.DeletePaths),
		zap.Bool("DryRun", params.DryRun),
		zap.String("ReportFile", params.ReportFile),
		zap.Bool("Verbose", params.Verbose),
		zap.Bool("Overwrite", params.Overwrite),
		zap.String("HashFunction", params.HashFunction),
		zap.Duration("StatusFrequency", params.StatusFrequency),
		zap.Bool("CoarseHashing", params.CoarseHashing),
		zap.Bool("EnableFilePprof", params.EnableFilePprof),
	)

	log.Info("Find Unique Files")
	log.Info("-----------------")

	log.Info("1. Indexing Files")

	scanner := newScanner()

	if err := localfs.ConcurrentWalk(params.Paths, scanner); err != nil {
		return nil, fmt.Errorf("cannot scan files for uniqueness, err= %w", err)
	}

	log.Info("2. Finding Duplicates")
	scanner.mergeAndIndex()

	log.Info("3. Building Report")
	r := scanner.report()

	return scanner, writeReportFile(params.ReportFile, r)
}
