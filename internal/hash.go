// Copyright 2019 Jeremy Edwards
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

package internal

import (
	"crypto/md5"
	"crypto/sha1"
	"crypto/sha256"
	"crypto/sha512"
	"fmt"
	"hash"
	"hash/crc32"
	"hash/crc64"
	"io"
	"os"
	"path/filepath"
	"strings"

	"github.com/pkg/errors"
)

var (
	crc64Table = crc64.MakeTable(crc64.ISO)
)

func newHashFromName(hashAlgorithmName string) hash.Hash {
	switch strings.Replace(strings.ToLower(hashAlgorithmName), "-", "", -1) {
	case "md5":
		return md5.New()
	case "sha1":
		return sha1.New()
	case "sha224":
		return sha256.New224()
	case "sha256":
		return sha256.New()
	case "sha384":
		return sha512.New384()
	case "sha512":
		return sha512.New()
	case "crc32":
		return crc32.NewIEEE()
	case "crc64":
		return crc64.New(crc64Table)
	default:
		return nil
	}
}

func hashFile(filename string, hashAlgorithmName string) (string, error) {
	h := newHashFromName(hashAlgorithmName)
	if h == nil {
		return "", errors.Errorf("algorithm %s is not supported", hashAlgorithmName)
	}
	return toHexString(hashFileAsBytes(filename, h))
}

func openFile(filename string) (*os.File, error) {
	absPath, err := filepath.Abs(filename)
	if err != nil {
		return nil, errors.Wrapf(err, "cannot get absolute file path for %s", filename)
	}
	f, err := os.Open(absPath)
	if err != nil {
		return nil, errors.Wrapf(err, "cannot open file, %s, for hashing", absPath)
	}
	return f, nil
}

func hashFileAsBytes(filename string, h hash.Hash) ([]byte, error) {
	if h == nil {
		return []byte{}, errors.Errorf("hash algorithm to be applied to %s was nil", filename)
	}
	f, err := openFile(filename)
	if err != nil {
		return []byte{}, err
	}
	// defer f.Close() is not called for performance reasons.
	res, err := computeHash(f, h)
	f.Close()
	return res, err
}

func computeHash(r io.Reader, h hash.Hash) ([]byte, error) {
	if _, err := io.Copy(h, r); err != nil {
		return []byte{}, errors.Wrap(err, "failed to hash data")
	}
	return h.Sum(nil), nil
}

func toHexString(hashCode []byte, err error) (string, error) {
	if err != nil {
		return "", err
	}
	return fmt.Sprintf("%x", hashCode), nil
}

func coarseHashFile(filename string, hashAlgorithmName string, bufferSize int) (string, error) {
	h := newHashFromName(hashAlgorithmName)
	if h == nil {
		return "", errors.Errorf("algorithm %s is not supported", hashAlgorithmName)
	}
	f, err := openFile(filename)
	if err != nil {
		return "", err
	}
	// defer f.Close() is not called for performance reasons.
	res, err := computeCoarseHash(f, bufferSize, h)
	f.Close()
	return toHexString(res, err)
}

func computeCoarseHash(r io.Reader, bufferSize int, h hash.Hash) ([]byte, error) {
	buf := make([]byte, bufferSize)
	readBytes, err := io.ReadFull(r, buf)
	if err != nil {
		return []byte{}, err
	} else if readBytes != bufferSize {
		return []byte{}, errors.Errorf("read %d bytes but wanted %d bytes", readBytes, bufferSize)
	}

	if _, err = h.Write(buf); err != nil {
		return []byte{}, err
	}
	return h.Sum(nil), nil
}
