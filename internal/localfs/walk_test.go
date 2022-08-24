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
	"fmt"
	"os"
	"sync"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/jeremyje/filetools/testdata"
)

type testWalkShard struct {
	files []string
	sync.RWMutex
}

func (s *testWalkShard) Accept(path string, info os.FileInfo, err error) error {
	s.Lock()

	s.files = append(s.files, path)
	s.Unlock()
	return nil
}

type testWalker struct {
	sync.RWMutex
	shards []*testWalkShard
}

func (s *testWalker) get() []string {
	allFiles := []string{}
	s.RLock()
	defer s.RUnlock()
	for _, shard := range s.shards {
		shard.RLock()
		files := shard.files
		shard.RUnlock()
		allFiles = append(allFiles, files...)
	}
	return allFiles
}

func (s *testWalker) NewWalkShard() func(string, os.FileInfo, error) error {
	tws := &testWalkShard{
		files: []string{},
	}

	s.Lock()
	s.shards = append(s.shards, tws)
	s.Unlock()

	return tws.Accept
}

func newTestWalker() *testWalker {
	return &testWalker{
		shards: []*testWalkShard{},
	}
}

func TestWalk(t *testing.T) {
	var testCases = []struct {
		paths []string
		want  []string
	}{
		{
			[]string{},
			[]string{},
		},
		{
			[]string{testdata.Get(t, "hasdupes")},
			[]string{
				testdata.Get(t, "hasdupes/a.1"), testdata.Get(t, "hasdupes/a.2"),
				testdata.Get(t, "hasdupes/a.3"), testdata.Get(t, "hasdupes/b.1"),
				testdata.Get(t, "hasdupes/b.2"), testdata.Get(t, "hasdupes/unique"),
			},
		},
		{
			[]string{testdata.Get(t, "hasdupes"), testdata.Get(t, "nodupes")},
			[]string{
				testdata.Get(t, "nodupes/empty"), testdata.Get(t, "nodupes/one"),
				testdata.Get(t, "nodupes/two"),
				testdata.Get(t, "hasdupes/a.1"), testdata.Get(t, "hasdupes/a.2"),
				testdata.Get(t, "hasdupes/a.3"), testdata.Get(t, "hasdupes/b.1"),
				testdata.Get(t, "hasdupes/b.2"), testdata.Get(t, "hasdupes/unique"),
			},
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("Serial %+v", tc.paths), func(t *testing.T) {
			t.Parallel()

			var m sync.Mutex
			actual := []string{}
			err := Walk(tc.paths, func(path string, info os.FileInfo, err error) error {
				m.Lock()
				actual = append(actual, path)
				m.Unlock()
				return nil
			})
			if err != nil {
				t.Fatal(err)
			}
			want := FromSlashList(tc.want)
			got := FromSlashList(actual)
			if diff := cmp.Diff(want, got); diff != "" {
				t.Errorf("Get mismatch (-want +got):\n%s", diff)
			}
		})

		ctc := tc
		t.Run(fmt.Sprintf("Concurrent %+v", ctc.paths), func(t *testing.T) {
			t.Parallel()
			s := newTestWalker()
			err := ConcurrentWalk(ctc.paths, s)
			if err != nil {
				t.Fatal(err)
			}
			want := FromSlashList(ctc.want)
			got := FromSlashList(s.get())
			if diff := cmp.Diff(want, got); diff != "" {
				t.Errorf("Get mismatch (-want +got):\n%s", diff)
			}
		})
	}
}
