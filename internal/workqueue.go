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
	"sync"
)

type listNode[TVal any] struct {
	val  TVal
	next *listNode[TVal]
}

type workQueue[TVal any] struct {
	begin *listNode[TVal]
	end   *listNode[TVal]
	sync.RWMutex
}

func (w *workQueue[TVal]) Submit(val TVal) {
	newNode := &listNode[TVal]{
		val:  val,
		next: nil,
	}
	w.Lock()
	if w.begin == nil {
		w.begin = newNode
		w.end = newNode
	} else {
		w.end.next = newNode
		w.end = newNode
	}
	w.Unlock()
}

func (w *workQueue[TVal]) Consume() (TVal, bool) {
	ok := false
	var v TVal
	w.Lock()
	if w.begin != nil {
		ok = true
		v = w.begin.val
		next := w.begin.next
		if w.begin == w.end {
			w.end = next
		}
		w.begin = next
	}
	w.Unlock()
	return v, ok
}

func newWorkQueue[TVal any]() *workQueue[TVal] {
	return &workQueue[TVal]{
		begin: nil,
		end:   nil,
	}
}
