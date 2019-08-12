package internal

import (
	"fmt"
	"github.com/pkg/errors"
	"os"
	"path/filepath"
	"sync"
)

type shardableWalkFunction interface {
	NewWalkShard() func(string, os.FileInfo, error) error
}

func filesOnly(f func(string, os.FileInfo, error) error) func(string, os.FileInfo, error) error {
	return func(path string, info os.FileInfo, err error) error {
		if info == nil {
			return err
		}
		if info.Mode()&os.ModeType == 0 {
			return f(path, info, err)
		}
		return err
	}
}

func shardedMultiwalk(paths []string, sharded shardableWalkFunction) error {
	paths = uniqueAndNonEmpty(paths)
	if len(paths) == 0 {
		return nil
	}
	for _, path := range paths {
		if !dirExists(path) {
			return errors.Errorf("%s is not a directory", path)
		}
	}
	chErr := make(chan error, 1)
	defer func() {
		close(chErr)
	}()

	var wg sync.WaitGroup
	for _, path := range paths {
		path := path
		wg.Add(1)
		go func() {
			defer wg.Done()
			f := filesOnly(sharded.NewWalkShard())
			walkErr := filepath.Walk(path, f)
			if walkErr != nil {
				select {
				case chErr <- walkErr:
				default:
					fmt.Printf("channel is full printing error, %s\n", walkErr)
				}
			}
		}()
	}
	wg.Wait()

	select {
	case err := <-chErr:
		return err
	default:
		return nil
	}
}

func multiwalk(paths []string, f func(string, os.FileInfo, error) error) error {
	paths = uniqueAndNonEmpty(paths)
	if len(paths) == 0 {
		return nil
	}
	chErr := make(chan error, 1)
	defer func() {
		close(chErr)
	}()

	filesOnlyF := filesOnly(f)
	var wg sync.WaitGroup
	for _, path := range paths {
		path := path
		wg.Add(1)
		go func() {
			defer wg.Done()
			walkErr := filepath.Walk(path, filesOnlyF)
			if walkErr != nil {
				select {
				case chErr <- walkErr:
				default:
					fmt.Printf("channel is full printing error, %s\n", walkErr)
				}
			}
		}()
	}
	wg.Wait()

	select {
	case err := <-chErr:
		return err
	default:
		return nil
	}
}
