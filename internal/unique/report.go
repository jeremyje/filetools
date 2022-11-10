// Copyright 2022 Jeremy Edwards
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

package unique

import (
	"encoding/csv"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	_ "embed"

	"github.com/jeremyje/filetools/internal/htmltemplate"
	pb "github.com/jeremyje/filetools/internal/unique/proto"
)

var (
	//go:embed template_report.html
	duplicateFileReportHTMLTemplate string
	//go:embed template_report.txt
	duplicateFileReportTXTTemplate string
)

func writeReportFile(name string, report *pb.DuplicateFileReport) error {
	if name == "" {
		return writeDuplicateFileReportText(os.Stdout, report)
	}

	ext := strings.TrimLeft(strings.TrimSpace(strings.ToLower(filepath.Ext(name))), ".")

	fp, err := os.Create(name)
	if err != nil {
		return fmt.Errorf("cannot create file %s, err= %w", name, err)
	}
	defer fp.Close()

	switch ext {
	case "csv":
		return writeDuplicateFileReportCSV(fp, report)
	case "html", "htm":
		return writeDuplicateFileReportHTML(fp, report)
	default:
		return writeDuplicateFileReportText(fp, report)
	}
}

func writeDuplicateFileReportCSV(w io.Writer, report *pb.DuplicateFileReport) error {
	cw := csv.NewWriter(w)
	if err := cw.Write([]string{"hash,name,size"}); err != nil {
		return fmt.Errorf("cannot write record to CSV, err= %w", err)
	}

	for _, dupFileSet := range report.GetDuplicates() {
		for _, currentSet := range dupFileSet.GetFile() {
			if err := cw.Write([]string{currentSet.GetHash(), currentSet.GetName(), fmt.Sprintf("%d", currentSet.GetSize())}); err != nil {
				return fmt.Errorf("cannot write record to CSV, err= %w", err)
			}
		}
	}
	return nil
}

func writeDuplicateFileReportHTML(w io.Writer, report *pb.DuplicateFileReport) error {
	return htmltemplate.Write(w, duplicateFileReportHTMLTemplate, report)
}

func writeDuplicateFileReportText(w io.Writer, report *pb.DuplicateFileReport) error {
	return htmltemplate.Write(w, duplicateFileReportTXTTemplate, report)
}
