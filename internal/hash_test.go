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
	"bytes"
	"crypto/md5"
	"crypto/sha1"
	"crypto/sha256"
	"crypto/sha512"
	"fmt"
	"hash"
	"hash/crc32"
	"hash/crc64"
	"os"
	"testing"

	xxhashOneOfOne "github.com/OneOfOne/xxhash"
	"github.com/cespare/xxhash/v2"
	"github.com/jeremyje/filetools/internal/localfs"
	"github.com/jeremyje/filetools/testdata"
	"github.com/pkg/errors"
	"github.com/stretchr/testify/assert"
)

const (
	kb = 1024
	mb = 1024 * 1024
)

func BenchmarkFastVsXxhash32(b *testing.B) {
	sizes := []int64{1, 4, 8, 128, kb, 4 * kb, 16 * kb, mb, 16 * mb}
	for _, size := range sizes {
		filename := mustFileOfLength(size)
		b.Run(fmt.Sprintf("xxhash32 x %d", size), func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				hashFile(filename, "xxhash32")
			}
		})
	}
	for _, size := range sizes {
		filename := mustFileOfLength(size)
		b.Run(fmt.Sprintf("uniqueHash x %d", size), func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				uniqueFile(filename)
			}
		})
	}
}

func BenchmarkHashFunctionsBySize(b *testing.B) {
	functions := []string{"md5", "sha1", "sha224", "sha256", "sha384", "sha512", "crc32", "crc64", "xxhash", "xxhash32", "xxhash64"}
	sizes := []int64{kb, 4 * kb, 16 * kb, mb, 16 * mb}
	for _, functionName := range functions {
		for _, size := range sizes {
			filename := mustFileOfLength(size)
			b.Run(fmt.Sprintf("%s x %d", functionName, size), func(b *testing.B) {
				for i := 0; i < b.N; i++ {
					hashFile(filename, functionName)
				}
			})
		}
	}
}

func BenchmarkHashFunctions(b *testing.B) {
	var testCases = []struct {
		hashFunctionName string
		hashFunction     hash.Hash
	}{
		{"md5", md5.New()},
		{"sha1", sha1.New()},
		{"sha224", sha256.New224()},
		{"sha256", sha256.New()},
		{"sha384", sha512.New384()},
		{"sha512", sha512.New()},
		{"crc32", crc32.NewIEEE()},
		{"crc64", crc64.New(crc64Table)},
		{"xxhash", xxhash.New()},
		{"xxhash32", xxhashOneOfOne.NewHash32()},
		{"xxhash64", xxhashOneOfOne.NewHash64()},
	}
	for _, tc := range testCases {
		tc := tc
		hundredChars := bytes.NewBufferString("0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789")
		b.Run(tc.hashFunctionName, func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				computeHash(hundredChars, tc.hashFunction)
			}
		})
	}
}

func mustFileOfLength(size int64) string {
	filename := fmt.Sprintf("../testdata/autogen/%d", size)
	err := os.MkdirAll("../testdata/autogen/", 0755)
	if err != nil {
		panic(err)
	}
	if localfs.FileExists(filename) {
		return filename
	}
	f, err := os.Create(filename)
	if err != nil {
		panic(err)
	}
	defer f.Close()
	for size > 0 {
		f.Write([]byte{byte(size)})
		size -= 1
	}
	return filename
}

func TestNewHashFromName(t *testing.T) {
	var testCases = []struct {
		name     string
		hashType hash.Hash
	}{
		{"MD5", md5.New()},
		{"MD-5", md5.New()},
		{"md-5", md5.New()},
		{"md5", md5.New()},
		{"sha1", sha1.New()},
		{"sha224", sha256.New224()},
		{"sha256", sha256.New()},
		{"sha384", sha512.New384()},
		{"sha512", sha512.New()},
		{"crc32", crc32.NewIEEE()},
		{"crc64", crc64.New(crc64Table)},
		{"xxhash", xxhash.New()},
		{"xxhash32", xxhashOneOfOne.NewHash32()},
		{"xxhash64", xxhashOneOfOne.NewHash64()},
		{"does-not-exist", nil},
		{"", nil},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("newHashFromName(%s)", tc.name), func(t *testing.T) {
			assert := assert.New(t)
			actual := newHashFromName(tc.name)
			if tc.hashType == nil {
				assert.Nil(actual)
			} else {
				assert.IsType(tc.hashType, actual)
			}
		})
	}
}

func TestHashFileErrorCases(t *testing.T) {
	assert := assert.New(t)
	// File Not Found
	hashCode, err := hashFile("does-not-exist", "md5")
	assert.Empty(hashCode)
	assert.Contains(err.Error(), "does-not-exist")
	// Algorithm Not Supported
	hashCode, err = hashFile("../testdata/hasdupes/a.1", "lol")
	assert.Empty(hashCode)
	assert.Contains(err.Error(), "lol")
}

func TestHashFileAsBytesErrorCases(t *testing.T) {
	assert := assert.New(t)

	hashCode, err := hashFileAsBytes("http://lol", nil)
	assert.Empty(hashCode)
	assert.Contains(err.Error(), "was nil")
}

type angryReadWriter struct {
}

func (a *angryReadWriter) Read([]byte) (int, error) {
	return 0, errors.New("i'm angry")
}
func (a *angryReadWriter) Write([]byte) (int, error) {
	return 0, errors.New("i'm angry")
}

func TestComputeHashErrorCases(t *testing.T) {
	assert := assert.New(t)

	hashCode, err := computeHash(&angryReadWriter{}, md5.New())
	assert.Empty(hashCode)
	assert.Contains(err.Error(), "failed to hash data")
}

func TestHashFile(t *testing.T) {
	var testCases = []struct {
		fileName      string
		hashAlgorithm string
		expected      string
	}{
		{testdata.Get(t, "hasdupes/a.1"), "MD5", "0cc175b9c0f1b6a831c399e269772661"},
		{testdata.Get(t, "hasdupes/a.1"), "sha1", "86f7e437faa5a7fce15d1ddcb9eaeaea377667b8"},
		{testdata.Get(t, "hasdupes/a.1"), "sha224", "abd37534c7d9a2efb9465de931cd7055ffdb8879563ae98078d6d6d5"},
		{testdata.Get(t, "hasdupes/a.1"), "sha256", "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb"},
		{testdata.Get(t, "hasdupes/a.1"), "sha384", "54a59b9f22b0b80880d8427e548b7c23abd873486e1f035dce9cd697e85175033caa88e6d57bc35efae0b5afd3145f31"},
		{testdata.Get(t, "hasdupes/a.1"), "sha512", "1f40fc92da241694750979ee6cf582f2d5d7d28e18335de05abc54d0560e0f5302860c652bf08d560252aa5e74210546f369fbbbce8c12cfc7957b2652fe9a75"},
		{testdata.Get(t, "hasdupes/a.1"), "crc32", "e8b7be43"},
		{testdata.Get(t, "hasdupes/a.1"), "crc64", "3420000000000000"},
		{testdata.Get(t, "hasdupes/b.1"), "MD5", "92eb5ffee6ae2fec3ad71c777531578f"},
		{testdata.Get(t, "hasdupes/a.1"), "xxhash", "d24ec4f1a98c6e5b"},
		{testdata.Get(t, "hasdupes/a.1"), "xxhash32", "550d7456"},
		{testdata.Get(t, "hasdupes/a.1"), "xxhash64", "d24ec4f1a98c6e5b"},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("hashFile(%s, %s)", tc.fileName, tc.hashAlgorithm), func(t *testing.T) {
			assert := assert.New(t)
			actual, err := hashFile(tc.fileName, tc.hashAlgorithm)
			assert.Nil(err)
			assert.Equal(tc.expected, actual)
		})
	}
}

func TestToHexString(t *testing.T) {
	assert := assert.New(t)
	expectedErr := errors.New("lol")
	strHash, err := toHexString([]byte{}, expectedErr)
	assert.Equal(expectedErr, err)
	assert.Empty(strHash)

	strHash, err = toHexString([]byte("012345"), nil)
	assert.Nil(err)
	assert.Equal("303132333435", strHash)
}
