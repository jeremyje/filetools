package internal

import (
	"fmt"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestStringList(t *testing.T) {
	var testCases = []struct {
		in       string
		expected []string
	}{
		{"", []string{}},
		{"abc", []string{"abc"}},
		{"abc,def", []string{"abc", "def"}},
		{"abc,def,", []string{"abc", "def"}},
		{"abc,,,def,", []string{"abc", "def"}},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("StringList(%s) => %v", tc.in, tc.expected), func(t *testing.T) {
			assert := assert.New(t)
			input := tc.in
			actual := StringList(&input)
			assert.ElementsMatch(tc.expected, actual)
		})
	}
}

func TestFileExists(t *testing.T) {
	var testCases = []struct {
		path     string
		expected bool
	}{
		{"does-not-exist", false},
		{"../testdata/hasdupes/a.1", true},
		{"../testdata/hasdupes/", false},
		{".", false},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("fileExists(%s) => %t", tc.path, tc.expected), func(t *testing.T) {
			assert := assert.New(t)
			actual := fileExists(tc.path)
			assert.Equal(tc.expected, actual)
		})
	}
}

func TestDirExists(t *testing.T) {
	var testCases = []struct {
		path     string
		expected bool
	}{
		{"does-not-exist", false},
		{"../testdata/hasdupes/a.1", false},
		{"../testdata/hasdupes/", true},
		{".", true},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("dirExists(%s) => %t", tc.path, tc.expected), func(t *testing.T) {
			assert := assert.New(t)
			actual := dirExists(tc.path)
			assert.Equal(tc.expected, actual)
		})
	}
}
