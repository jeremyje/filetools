package internal

import (
	"fmt"
	"github.com/stretchr/testify/assert"
	"os"
	"sync"
	"testing"
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
			[]string{"."},
			[]string{
				"hash.go", "hash_test.go", "multiwalk.go", "multiwalk_test.go",
				"report.go", "similar.go", "similar_test.go", "unique.go",
				"unique_test.go", "util.go", "util_test.go",
			},
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
			assert.ElementsMatch(tc.expected, actual)
		})
	}
}
