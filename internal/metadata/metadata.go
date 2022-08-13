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

// Package metadata extracts important information from a file. This is basically an extended version of os.Stat().
package metadata

import (
	"fmt"
	"mime"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/dhowden/tag"
	pb "github.com/jeremyje/filetools/internal/metadata/proto"
	"github.com/pillash/mp4util"
	tajtiattila "github.com/tajtiattila/metadata"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/timestamppb"
)

func StatFromFilepath(path string) (*pb.FileMetadata, error) {
	stat, err := os.Stat(path)
	if err != nil {
		return nil, err
	}

	return StatFromFileWalk(path, stat)
}

func StatFromFileWalk(path string, info os.FileInfo) (*pb.FileMetadata, error) {
	fileStat := fileInfoToStatProto(path, info)

	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}

	defer f.Close()
	mediaMD := &pb.FileMetadataMedia{}

	tagMD, err := tag.ReadFrom(f)
	if err == nil {
		mediaMD.Format = string(tagMD.Format())
		mediaMD.FileType = string(tagMD.FileType())
		raws := map[string]string{}
		for k, v := range tagMD.Raw() {
			raws[k] = fmt.Sprintf("%v", v)
		}
		mediaMD.Raw = raws
	}

	f.Seek(0, 0)

	if strings.Contains(fileStat.GetMimeType(), "mp4") {
		seconds, err := mp4util.Duration(path)
		if err == nil {
			mediaMD.Duration = durationpb.New(time.Second * time.Duration(seconds))
		}
	}

	f.Seek(0, 0)

	tMD, err := tajtiattila.Parse(f)
	if err == nil {
		attrs := map[string]string{}
		for k, v := range tMD.Attr {
			attrs[k] = v
		}

		mediaMD.Attributes = attrs
		mediaMD.Orientation = int32(tMD.Orientation)
		mediaMD.CreatedTimestamp = timestamppb.New(tMD.DateTimeCreated.Time)
		mediaMD.OriginalTimestamp = timestamppb.New(tMD.DateTimeOriginal.Time)
	}

	return &pb.FileMetadata{
		FileStat: fileStat,
		Media:    mediaMD,
	}, nil
}

func fileInfoToStatProto(path string, info os.FileInfo) *pb.FileStat {
	fullPath, err := filepath.Abs(path)
	if err != nil {
		fullPath = path
	}
	ext := ext(info.Name())

	mimeType := mime.TypeByExtension(ext)
	if len(mimeType) == 0 {
		ext = filepath.Ext(info.Name())
		mimeType = mime.TypeByExtension(ext)
	}

	return &pb.FileStat{
		Name:          info.Name(),
		Size:          info.Size(),
		Mode:          uint32(info.Mode()),
		ModTime:       timestamppb.New(info.ModTime()),
		IsDir:         info.IsDir(),
		FullPath:      fullPath,
		MimeType:      mimeType,
		FileExtension: ext,
	}
}

func ext(fileName string) string {
	if len(fileName) == 0 {
		return ""
	}
	if fileName[0] == '.' {
		return ""
	}
	parts := strings.SplitN(fileName, ".", 2)
	if len(parts) == 1 {
		return ""
	}
	return "." + parts[1]
}
