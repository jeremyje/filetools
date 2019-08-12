package internal

import (
	"github.com/pkg/errors"
	"io"
	"os"
	"text/template"
)

const fileOpenOverwrite = os.O_RDWR | os.O_CREATE
const fileOpenDontOverwrite = fileOpenOverwrite | os.O_EXCL
const defaultPermissions = 0755

const duplicateFileReportTemplate = `<!DOCTYPE html>
<html>
	<head>
		<meta charset="UTF-8">
		<title>{{.Title}}</title>
		<style>
		body {
			font-family: Arial;
			font-size: 12px;
		}
		pre {
			font-family: Consolas, monospace;
			font-size: 12px;
		}
		table {
			table-layout: fixed;
		}
		.odd {
			background: #DDDDDD;
		}
		</style>
	</head>
	<body>
		<table>
		{{ range $fileSize, $duplicateFileSet := .Duplicates -}}
		{{- range $duplicateFileSet.Names -}}
		<tr><td>{{ $duplicateFileSet.Size }}</td><td>{{ . }}</td></tr>
		{{ end }}
		{{- end -}}
		</table>
	</body>
</html>`

// FileTable is a table of files
type FileTable struct {
	Title    string
	FileItem []FileItem
}

// FileItem is a descriptor for a file.
type FileItem struct {
	Name  string
	Size  int64
	Style string
}

func openFileForWrite(filename string, overwrite bool) (*os.File, error) {
	openArgs := fileOpenDontOverwrite
	if overwrite {
		openArgs = fileOpenOverwrite
	}
	return os.OpenFile(filename, openArgs, defaultPermissions)
}

func writeReport(w io.Writer, templateText string, arg interface{}) error {
	t, err := template.New("report").Funcs(template.FuncMap{
		"sizeString": sizeString,
		"oddEven":    newEvenOdd().next,
	}).Parse(templateText)
	if err != nil {
		return errors.Wrap(err, "cannot parse HTML template")
	}
	return t.Execute(w, arg)
}
