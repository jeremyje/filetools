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

package metadata

import (
	"fmt"
	"os"
	"testing"

	"github.com/google/go-cmp/cmp"
	pb "github.com/jeremyje/filetools/internal/metadata/proto"
	"google.golang.org/protobuf/testing/protocmp"
)

func TestMetadata(t *testing.T) {
	var testCases = []struct {
		path string
		want *pb.FileStat
	}{
		{
			path: "testdata/testfile.txt",
			want: &pb.FileStat{
				Name:          "testfile.txt",
				Size:          12,
				Mode:          420,
				IsDir:         false,
				MimeType:      "text/plain; charset=utf-8",
				FileExtension: ".txt",
			},
		},
		{
			path: "testdata/testfile.txt.txt",
			want: &pb.FileStat{
				Name:          "testfile.txt.txt",
				Size:          16,
				Mode:          420,
				IsDir:         false,
				MimeType:      "text/plain; charset=utf-8",
				FileExtension: ".txt",
			},
		},
		{
			path: "testdata/testfile.tar",
			want: &pb.FileStat{
				Name:          "testfile.tar",
				Size:          10240,
				Mode:          420,
				IsDir:         false,
				MimeType:      "application/x-tar",
				FileExtension: ".tar",
			},
		},
		{
			path: "testdata/testfile.tar.gz",
			want: &pb.FileStat{
				Name:          "testfile.tar.gz",
				Size:          152,
				Mode:          420,
				IsDir:         false,
				MimeType:      "application/x-compressed-tar",
				FileExtension: ".tar.gz",
			},
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("fileInfoToStatProto(%s)", tc.path), func(t *testing.T) {
			t.Parallel()
			stat, err := os.Stat(tc.path)
			if err != nil {
				t.Fatalf("cannot stat file, %s", err)
			}

			got := fileInfoToStatProto(tc.path, stat)

			if diff := cmp.Diff(tc.want, got, protocmp.Transform(), protocmp.IgnoreFields(tc.want, "full_path", "mod_time")); diff != "" {
				t.Errorf("Get mismatch (-want +got):\n%s", diff)
			}
		})

		t.Run(fmt.Sprintf("StatFromFilepath(%s)", tc.path), func(t *testing.T) {
			t.Parallel()
			got, err := StatFromFilepath(tc.path)
			if err != nil {
				t.Fatal(err)
			}

			if diff := cmp.Diff(tc.want, got.GetFileStat(), protocmp.Transform(), protocmp.IgnoreFields(tc.want, "full_path", "mod_time")); diff != "" {
				t.Errorf("Get mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

func TestExt(t *testing.T) {
	var testCases = []struct {
		path string
		want string
	}{
		{
			path: "",
			want: "",
		},
		{
			path: ".",
			want: "",
		},
		{
			path: "/",
			want: "",
		},
		{
			path: "noext",
			want: "",
		},
		{
			path: "file.ext",
			want: ".ext",
		},
		{
			path: "testdata/testfile.txt",
			want: ".txt",
		},
		{
			path: "testdata/testfile.txt.txt",
			want: ".txt.txt",
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(tc.path, func(t *testing.T) {
			t.Parallel()

			got := ext(tc.path)
			if diff := cmp.Diff(tc.want, got); diff != "" {
				t.Errorf("ext() mismatch (-want +got):\n%s", diff)
			}
		})
	}
}
