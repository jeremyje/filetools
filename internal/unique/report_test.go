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
	"bytes"
	_ "embed"
	"testing"

	"github.com/google/go-cmp/cmp"
	pb "github.com/jeremyje/filetools/internal/unique/proto"
)

var (
	//go:embed test_report.html
	testReportHTML string
	//go:embed test_report.txt
	testReportTXT string
)

func TestReport(t *testing.T) {
	buf := bytes.NewBufferString("")
	r := &pb.DuplicateFileReport{
		Title:         "Duplicate File Report",
		DeleteEnabled: true,
		Duplicates: []*pb.DuplicateFileReport_DuplicateFileSet{
			{
				Size: 5,
				Hash: "one",
				File: []*pb.FileSummary{
					{
						Name:         "one",
						Size:         5,
						Hash:         "one",
						ShouldDelete: false,
					},
					{
						Name:         "two",
						Size:         5,
						Hash:         "one",
						ShouldDelete: true,
					},
					{
						Name:         "three",
						Size:         5,
						Hash:         "one",
						ShouldDelete: true,
					},
				},
			},
			{
				Size: 4,
				Hash: "two",
				File: []*pb.FileSummary{
					{
						Name: "four",
						Size: 4,
						Hash: "two",
					},
					{
						Name: "five",
						Size: 4,
						Hash: "two",
					},
				},
			},
		},
	}

	err := writeDuplicateFileReportHTML(buf, r)
	compareText(t, testReportHTML, buf.String(), err)

	buf.Reset()
	err = writeDuplicateFileReportText(buf, r)
	compareText(t, testReportTXT, buf.String(), err)
}

func compareText(tb testing.TB, want string, got string, err error) {
	if err != nil {
		tb.Error(err)
	}
	if diff := cmp.Diff(want, got); diff != "" {
		tb.Error(diff)
		tb.Errorf("Wanted\n%s", want)
		tb.Errorf("Got\n%s", got)
	}
}
