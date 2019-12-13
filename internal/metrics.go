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
	"fmt"
	"sync/atomic"
	"time"
)

type counter struct {
	val   *int64
	label string
}

func (c *counter) String() string {
	return fmt.Sprintf("%s %v", c.label, c.value())
}

func (c *counter) inc() {
	c.incBy(1)
}

func (c *counter) set(val int64) {
	atomic.StoreInt64(c.val, val)
}

func (c *counter) incBy(val int64) {
	atomic.AddInt64(c.val, val)
}

func (c *counter) value() int64 {
	return atomic.LoadInt64(c.val)
}

func newCounter(label string) *counter {
	return &counter{
		label: label,
		val:   new(int64),
	}
}

type measure struct {
	start    time.Time
	label    string
	subs     []*measure
	isdone   bool
	duration time.Duration
}

func newMeasure(label string) *measure {
	return &measure{
		start: time.Now(),
		label: label,
		subs:  []*measure{},
	}
}

func (m *measure) done() {
	if m.isdone {
		return
	}
	m.doneAt(time.Now())
}

func (m *measure) doneAt(end time.Time) {
	if m.isdone {
		return
	}
	m.isdone = true
	for _, sub := range m.subs {
		sub.doneAt(end)
	}
	m.duration = end.Sub(m.start)
	m.print()
}

func (m *measure) print() {
	fmt.Printf("- %s\n", m.String())
}

func (m *measure) sub(label string) *measure {
	fmt.Printf("- %s\n", label)
	sub := newMeasure(label)
	m.subs = append(m.subs, sub)
	return sub
}

func (m *measure) String() string {
	return fmt.Sprintf("%s: %s", m.label, m.duration)
}
