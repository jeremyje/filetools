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
	"strings"
	"sync/atomic"

	"github.com/dustin/go-humanize"
)

func uniqueAndNonEmpty(items []string) []string {
	m := map[string]interface{}{}
	for _, item := range items {
		if len(item) > 0 {
			m[item] = nil
		}
	}
	unique := []string{}
	for item := range m {
		unique = append(unique, item)
	}
	return unique
}

// StringList removes all empty and duplicate entries from a comma separated list of strings.
func StringList(flagValue *string) []string {
	if len(*flagValue) == 0 {
		return []string{}
	}
	return uniqueAndNonEmpty(strings.SplitN(*flagValue, ",", -1))
}

// Check reports errors to stdout.
func Check(err error) {
	if err != nil {
		fmt.Printf("%s", err)
	}
}

func sizeString(size int64) string {
	return humanize.IBytes(uint64(size))
}

type evenOdd struct {
	counter *uint64
}

func (eo *evenOdd) next() bool {
	old := atomic.AddUint64(eo.counter, 1)
	return old%2 == 1
}

func newEvenOdd() *evenOdd {
	z := uint64(0)
	return &evenOdd{
		counter: &z,
	}
}
