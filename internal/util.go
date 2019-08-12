package internal

import (
	"fmt"
	"github.com/dustin/go-humanize"
	"os"
	"strings"
	"sync/atomic"
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

func fileExists(path string) bool {
	info, err := os.Stat(path)
	if err != nil {
		return false
	}
	return !info.IsDir()
}

func dirExists(path string) bool {
	info, err := os.Stat(path)
	if err != nil {
		return false
	}
	return info.IsDir()
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
