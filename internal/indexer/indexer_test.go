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

package indexer

import (
	"io/fs"
	"path/filepath"
	"testing"

	"github.com/blevesearch/bleve/v2"
	"github.com/jeremyje/filetools/internal/metadata"
	"github.com/jeremyje/filetools/testdata"
)

func TestMetadataMedia(t *testing.T) {
	dir := t.TempDir()
	indexFileName := filepath.Join(dir, "indexer")
	idx, err := New(indexFileName)
	if err != nil {
		t.Fatal(err)
	}

	testdir := testdata.GetDirectory(t)
	if err := filepath.WalkDir(testdir, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if !d.IsDir() {
			md, err := metadata.StatFromFilepath(path)
			if err != nil {
				return err
			}
			return idx.Put(md)
		} else {
			return nil
		}
	}); err != nil {
		t.Fatal(err)
	}

	result, err := idx.index.Search(bleve.NewSearchRequest(bleve.NewMatchQuery("file")))
	if err != nil {
		t.Fatal(err)
	}
	t.Log(result)
	for k, v := range result.Hits {
		t.Logf("k:%v v:%v", k, v)
	}
	if result.Hits.Len() == 0 {
		t.Fatal("no hits")
	}
}
