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
	"fmt"

	pb "github.com/jeremyje/filetools/internal/metadata/proto"
	"go.etcd.io/bbolt"
	"google.golang.org/protobuf/proto"
)

var (
	fileBucketName = []byte("files")
)

type DB struct {
	db *bbolt.DB
}

func (db *DB) Put(val *pb.FileMetadata) error {
	if db.db == nil {
		return fmt.Errorf("cannot add file, '%s', because the database is closed", val.GetFileUri().GetId())
	}
	bsVal, err := proto.Marshal(val)
	if err != nil {
		return err
	}
	tx, err := db.db.Begin(true)
	if err != nil {
		return err
	}
	bkt, err := tx.CreateBucketIfNotExists(fileBucketName)
	if err != nil {
		return err
	}
	return bkt.Put([]byte(val.GetFileUri().GetId()), bsVal)
}

func (db *DB) Close() error {
	d := db.db
	db.db = nil
	return d.Close()
}

func NewDB(name string) (*DB, error) {
	db, err := bbolt.Open(name, 0644, nil)
	if err != nil {
		return nil, err
	}
	return &DB{
		db: db,
	}, nil
}
