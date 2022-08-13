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
	"time"

	"github.com/google/go-cmp/cmp"
	pb "github.com/jeremyje/filetools/internal/metadata/proto"
	"github.com/jeremyje/filetools/testdata"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/durationpb"
)

func TestMetadataMedia(t *testing.T) {
	var testCases = []struct {
		path string
		want *pb.FileMetadataMedia
	}{
		{
			path: testdata.Get(t, "examples/file_example_favicon.ico"),
			want: &pb.FileMetadataMedia{
				Attributes: map[string]string{},
			},
		},
		{
			path: testdata.Get(t, "examples/file_example_GIF_500kB.gif"),
			want: &pb.FileMetadataMedia{
				Attributes: map[string]string{},
			},
		},
		{
			path: testdata.Get(t, "examples/file_example_JPG_100kB.jpg"),
			want: &pb.FileMetadataMedia{
				Attributes: map[string]string{},
			},
		},
		{
			path: testdata.Get(t, "examples/file_example_MP4_480_1_5MG.mp4"),
			want: &pb.FileMetadataMedia{
				Attributes: map[string]string{
					"DateTimeCreated": "2015-08-07T09:13:02Z",
				},
				Format:   "MP4",
				Duration: durationpb.New(time.Second * time.Duration(30)),
			},
		},
		{
			path: testdata.Get(t, "examples/file_example_PNG_500kB.png"),
			want: &pb.FileMetadataMedia{
				Attributes: map[string]string{},
			},
		},
		{
			path: testdata.Get(t, "examples/file_example_TIFF_1MB.tiff"),
			want: &pb.FileMetadataMedia{
				Attributes: map[string]string{},
			},
		},
		{
			path: testdata.Get(t, "examples/file_example_WEBP_50kB.webp"),
			want: &pb.FileMetadataMedia{
				Attributes: map[string]string{},
			},
		},
		{
			path: testdata.Get(t, "examples/file_example_MP3_700KB.mp3"),
			want: &pb.FileMetadataMedia{
				Attributes: map[string]string{},
				FileType:   "MP3",
				Format:     "ID3v2.3",
				Raw: map[string]string{
					"TALB": "YouTube Audio Library",
					"TCON": "Cinematic",
					"TIT2": "Impact Moderato",
					"TPE1": "Kevin MacLeod",
				},
			},
		},
		{
			path: testdata.Get(t, "examples/file_example_OOG_1MG.ogg"),
			want: &pb.FileMetadataMedia{
				Attributes: map[string]string{},
				FileType:   "OGG",
				Format:     "VORBIS",
				Raw: map[string]string{
					"album":  "YouTube Audio Library",
					"artist": "Kevin MacLeod",
					"genre":  "Cinematic",
					"title":  "Impact Moderato",
					"vendor": "Xiph.Org libVorbis I 20120203 (Omnipresent)",
				},
			},
		},
		{
			path: testdata.Get(t, "examples/file_example_SVG_20kB.svg"),
			want: &pb.FileMetadataMedia{
				Attributes: map[string]string{},
			},
		},
		{
			path: testdata.Get(t, "examples/file_example_WAV_1MG.wav"),
			want: &pb.FileMetadataMedia{
				Attributes: map[string]string{},
			},
		},
	}

	for _, tc := range testCases {
		tc := tc
		t.Run(tc.path, func(t *testing.T) {
			t.Parallel()
			got, err := StatFromFilepath(tc.path)
			if err != nil {
				t.Fatal(err)
			}

			if diff := cmp.Diff(tc.want, got.GetMedia(), protocmp.Transform(), protocmp.IgnoreFields(tc.want, "created_timestamp", "original_timestamp")); diff != "" {
				t.Errorf("Get mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

func TestMetadata(t *testing.T) {
	var testCases = []struct {
		path string
		want *pb.FileStat
	}{
		{
			path: testdata.Get(t, "examples/testfile.txt"),
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
			path: testdata.Get(t, "examples/testfile.txt.txt"),
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
			path: testdata.Get(t, "examples/testfile.tar"),
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
			path: testdata.Get(t, "examples/testfile.tar.gz"),
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
