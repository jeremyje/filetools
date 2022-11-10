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

package htmltemplate

import (
	"html/template"
	"io"
	"strings"
	"sync/atomic"

	_ "embed"

	"github.com/dustin/go-humanize"
	"github.com/pkg/errors"
)

var (
	//go:embed duplicate_report.html
	DuplicateFileReportTemplate string
)

func Write(w io.Writer, templateText string, arg interface{}) error {
	t, err := template.New("report").Funcs(template.FuncMap{
		"sizeString": sizeString,
		"fileURI":    fileURI,
		"oddEven":    newEvenOdd().next,
	}).Parse(templateText)
	if err != nil {
		return errors.Wrap(err, "cannot parse HTML template")
	}
	return t.Execute(w, arg)
}

func sizeString(size int64) string {
	return humanize.IBytes(uint64(size))
}

func fileURI(name string) string {
	return strings.ReplaceAll(name, "\\", "/")
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
