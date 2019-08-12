package internal

import (
	"fmt"
	"log"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

var (
	alphaNumericRegex *regexp.Regexp = regexp.MustCompile("[^a-zA-Z0-9]+")
)

// SimilarParams are the parameters for finding similarly named files.
type SimilarParams struct {
	Paths       []string
	ClearTokens []string
}

// Similar finds similarly named files in a directory structure.
func Similar(p *SimilarParams) {
	matches := findSimilarFiles(p)
	report(matches)
}

func report(matches map[string][]string) {
	fmt.Println("Same files")
	for _, m := range matches {
		for _, v := range m {
			fmt.Printf("%s\n", v)
		}
		fmt.Printf("\n")
	}
}

func findSimilarFiles(p *SimilarParams) map[string][]string {
	clearTokens := p.ClearTokens
	fileTable := make(map[string][]string)
	err := multiwalk(p.Paths, func(path string, info os.FileInfo, err error) error {
		if err == nil {
			normName := normalize(path, clearTokens)
			arr, ok := fileTable[normName]
			if ok {
				fileTable[normName] = append(arr, path)
			} else {
				fileTable[normName] = []string{path}
			}
		} else {
			log.Printf("ERROR: Can't read %s, %s", path, err)
		}
		return nil
	})
	if err != nil {
		fmt.Printf("%s", err)
		return map[string][]string{}
	}

	// Sanitize

	result := make(map[string][]string)
	for k, v := range fileTable {
		if len(v) > 1 {
			result[k] = v
		}
	}
	return result
}

func normalize(path string, clearTokens []string) string {
	p := filepath.Base(path)
	p = strings.TrimSuffix(p, filepath.Ext(p))
	for _, v := range clearTokens {
		p = strings.Replace(p, v, "", -1)
	}
	p = alphaNumericRegex.ReplaceAllString(p, "")
	return p
}
