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
	"github.com/stretchr/testify/assert"
	"sync"
	"testing"
	"time"
)

func TestCounter(t *testing.T) {
	assert := assert.New(t)
	c := newCounter("test")
	assert.Equal(int64(0), c.value())
	c.inc()
	assert.Equal(int64(1), c.value())
	c.incBy(10)
	assert.Equal(int64(11), c.value())
	var wg sync.WaitGroup
	incs := 100
	for incs > 0 {
		wg.Add(1)
		go func() {
			c.inc()
			wg.Done()
		}()
		incs--
	}
	wg.Wait()
	assert.Equal(int64(111), c.value())
	assert.Equal("test 111", c.String())
}

func TestMeasure(t *testing.T) {
	m := newMeasure("root")
	m1 := m.sub("first")
	m2 := m.sub("second")
	m3 := m.sub("third")
	m1.done()
	time.Sleep(10 * time.Millisecond)
	m.done()

	if m.duration < m1.duration {
		t.Errorf("Expected (m) %s >= (m1) %s", m, m1)
	}
	if m.duration < m2.duration {
		t.Errorf("Expected (m) %s >= (m2) %s", m, m2)
	}
	if m.duration < m3.duration {
		t.Errorf("Expected (m) %s >= (m3) %s", m, m3)
	}
	if m2.duration < m1.duration {
		t.Errorf("Expected (m2) %s >= (m1) %s", m2, m1)
	}
	if m3.duration < m1.duration {
		t.Errorf("Expected (m3) %s >= (m1) %s", m3, m1)
	}
}
