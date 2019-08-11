package internal

import (
	"fmt"
	"os"
	"strings"
)

func StringList(flagValue *string) []string {
	if len(*flagValue) == 0 {
		return []string{}
	}
	m := map[string]interface{}{}
	for _, item := range strings.SplitN(*flagValue, ",", -1) {
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

func Check(err error) {
	if err != nil {
		fmt.Printf("%s", err)
	}
}
