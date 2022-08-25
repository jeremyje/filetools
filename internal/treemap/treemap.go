// Copyright 2020 Jeremy Edwards
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

// Package treemap renders a treemap of the file system.
package treemap

import (
	"log"
	"os"
	"path/filepath"
	"sync"

	"github.com/dustin/go-humanize"
	"github.com/jeremyje/filetools/internal/localfs"
)

// Params are the parameters for filesystem treemap.
type Params struct {
	Paths   []string
	MinSize int64
	Output  string
}

func Run(p *Params) {
	results, err := treemap(p)
	report(results, err)
}

func treemap(p *Params) (*Node, error) {
	nf := newNewFactory(p.MinSize)
	err := localfs.Walk(p.Paths, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			log.Printf("ERROR: Can't read %s, %s", path, err)
			return nil
		}
		nf.add(path, info)

		return nil
	})
	if err != nil {
		return nil, err
	}
	return nf.root, nil
}

func report(root *Node, err error) {
	log.Printf("%s %s", root.Name(), humanize.Bytes(uint64(root.Size())))
}

func newNode(name string, info os.FileInfo) *Node {
	size := int64(0)
	if info != nil {
		size = info.Size()
	}
	return &Node{
		name:  name,
		size:  size,
		isDir: info == nil,
	}
}

type nodeFactory struct {
	sync.Mutex
	root    *Node
	m       map[string]*Node
	minSize int64
}

func (nf *nodeFactory) add(path string, info os.FileInfo) {
	nf.Lock()
	defer nf.Unlock()
	nn := newNode(path, info)
	dir := filepath.Dir(path)
	base, ok := nf.m[dir]
	if !ok {
		paths := localfs.ExplodePath(path)
		parentParent := nf.root
		for i := len(paths) - 2; i > 0; i-- {
			parent := paths[i]
			base, ok = nf.m[parent]
			if !ok {
				base = newNode(parent, nil)
				nf.m[parent] = base
				parentParent.addChild(base)
			}
			parentParent = base
		}
	}

	if info.Size() >= nf.minSize {
		base.addChild(nn)
	} else {
		base.size += info.Size()
	}
}

func newNewFactory(minSize int64) *nodeFactory {
	root := newNode("/", nil)
	return &nodeFactory{
		root:    root,
		m:       map[string]*Node{"/": root},
		minSize: minSize,
	}
}

type Node struct {
	name     string
	size     int64
	isDir    bool
	children []*Node
	parent   *Node
}

func (n *Node) addChild(nn *Node) {
	n.children = append(n.children, nn)
	nn.parent = n
}

func (n *Node) Name() string {
	return n.name
}

func (n *Node) FileCount() int64 {
	count := int64(0)
	if !n.isDir {
		count++
	}
	for _, c := range n.children {
		count += c.FileCount()
	}

	return count
}
func (n *Node) Size() int64 {
	ts := n.size
	for _, s := range n.children {
		ts += s.Size()
	}
	return ts
}
