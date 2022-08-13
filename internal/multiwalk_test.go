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
	"os"
	"sync"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestMultiWalk(t *testing.T) {
	var testCases = []struct {
		paths    []string
		expected []string
	}{
		{
			[]string{},
			[]string{},
		},
		{
			[]string{"../testdata/hasdupes"},
			[]string{
				"../testdata/hasdupes/a.1", "../testdata/hasdupes/a.2",
				"../testdata/hasdupes/a.3", "../testdata/hasdupes/b.1",
				"../testdata/hasdupes/b.2", "../testdata/hasdupes/unique",
			},
		},
		{
			[]string{"../testdata/hasdupes", "../testdata/nodupes"},
			[]string{
				"../testdata/nodupes/empty", "../testdata/nodupes/one",
				"../testdata/nodupes/two",
				"../testdata/hasdupes/a.1", "../testdata/hasdupes/a.2",
				"../testdata/hasdupes/a.3", "../testdata/hasdupes/b.1",
				"../testdata/hasdupes/b.2", "../testdata/hasdupes/unique",
			},
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("multiwalk(%+v)", tc.paths), func(t *testing.T) {
			assert := assert.New(t)
			var m sync.Mutex
			actual := []string{}
			err := multiwalk(tc.paths, func(path string, info os.FileInfo, err error) error {
				m.Lock()
				actual = append(actual, path)
				m.Unlock()
				return nil
			})
			assert.Nil(err)
			assert.ElementsMatch(fromSlashList(tc.expected), fromSlashList(actual))
		})
	}
}
