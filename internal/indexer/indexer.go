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

// Package indexer stores file information in a full text search database.
package indexer

import (
	"github.com/blevesearch/bleve/v2"
	"github.com/blevesearch/bleve/v2/mapping"
	pb "github.com/jeremyje/filetools/internal/metadata/proto"
)

type Indexer struct {
	name    string
	mapping *mapping.IndexMappingImpl
	index   bleve.Index
}

func (indexer *Indexer) Put(val *pb.FileMetadata) error {
	return indexer.index.Index(val.FileUri.GetId(), val)
}

func New(name string) (*Indexer, error) {
	mapping := bleve.NewIndexMapping()
	index, err := bleve.New(name, mapping)
	if err != nil {
		return nil, err
	}
	return &Indexer{
		name:    name,
		mapping: mapping,
		index:   index,
	}, nil
}
