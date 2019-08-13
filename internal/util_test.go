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
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestUniqueAndNonEmpty(t *testing.T) {
	var testCases = []struct {
		in       []string
		expected []string
	}{
		{[]string{}, []string{}},
		{[]string{""}, []string{}},
		{[]string{"abc"}, []string{"abc"}},
		{[]string{"abc", "abc", "abc"}, []string{"abc"}},
		{[]string{"abc", "def", "abc"}, []string{"abc", "def"}},
		{[]string{"abc", "def", "abc", "", "def", "", "", ""}, []string{"abc", "def"}},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("uniqueAndNonEmpty(%s) => %v", tc.in, tc.expected), func(t *testing.T) {
			assert := assert.New(t)
			actual := uniqueAndNonEmpty(tc.in)
			assert.ElementsMatch(tc.expected, actual)
		})
	}
}

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

func TestSizeString(t *testing.T) {
	var testCases = []struct {
		size     int64
		expected string
	}{
		{0, "0 B"},
		{1024, "1.0 KiB"},
		{1024 * 1024, "1.0 MiB"},
		{1024 * (1024 * 1.5), "1.5 MiB"},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("sizeString(%d) => %s", tc.size, tc.expected), func(t *testing.T) {
			assert := assert.New(t)
			actual := sizeString(tc.size)
			assert.Equal(tc.expected, actual)
		})
	}
}

func TestEvenOdd(t *testing.T) {
	assert := assert.New(t)
	eo := newEvenOdd()
	assert.True(eo.next())
	assert.False(eo.next())
	assert.True(eo.next())
}

func TestCheck(t *testing.T) {
	Check(nil)
	Check(errors.New("lol"))
}
