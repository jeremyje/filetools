package main

import (
	"flag"
	"github.com/jeremyje/filetools/internal"
)

var (
	pathFlag       = flag.String("path", "", "Comma separated list of directory paths to scan for similar files.")
	clearTokenFlag = flag.String("clear", "", "Clear tokens")
)

func main() {
	flag.Parse()
	internal.Similar(fromFlags())
}

func fromFlags() *internal.SimilarParams {
	return &internal.SimilarParams{
		Paths:       internal.StringList(pathFlag),
		ClearTokens: internal.StringList(clearTokenFlag),
	}
}
