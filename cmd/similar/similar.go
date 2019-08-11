package main

import (
	"flag"
	"github.com/jeremyje/filetools/internal"
)

var (
	pathFlag       = flag.String("path", "", "Path of directory tree to scan for duplicates.")
	clearTokenFlag = flag.String("clear", "", "Clear tokens")
)

func main() {
	flag.Parse()
	internal.Similar(fromFlags())
}

func fromFlags() *internal.SimilarParams {
	return &internal.SimilarParams{
		Path:        *pathFlag,
		ClearTokens: internal.StringList(clearTokenFlag),
	}
}
