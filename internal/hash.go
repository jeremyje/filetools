package internal

import (
	"crypto/md5"
	"crypto/sha1"
	"crypto/sha256"
	"crypto/sha512"
	"fmt"
	"github.com/pkg/errors"
	"hash"
	"hash/crc32"
	"hash/crc64"
	"io"
	"os"
	"path/filepath"
	"strings"
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

func hashFileAsBytes(filename string, h hash.Hash) ([]byte, error) {
	absPath, err := filepath.Abs(filename)
	if err != nil {
		return []byte{}, errors.Wrapf(err, "cannot get absolute file path for %s", filename)
	}
	f, err := os.Open(absPath)
	if err != nil {
		return []byte{}, errors.Wrapf(err, "cannot open file, %s, for hashing", absPath)
	}
	// defer f.Close() is not called for performance reasons.
	res, err := computeHash(f, h)
	f.Close()
	return res, err
}

func computeHash(w io.ReadWriter, h hash.Hash) ([]byte, error) {
	if _, err := io.Copy(h, w); err != nil {
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
