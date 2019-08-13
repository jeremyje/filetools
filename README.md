# File Tools

[![GoDoc](https://godoc.org/github.com/jeremyje/filetools?status.svg)](https://godoc.org/github.com/jeremyje/filetools)
[![Go Report Card](https://goreportcard.com/badge/github.com/jeremyje/filetools)](https://goreportcard.com/report/github.com/jeremyje/filetools)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://github.com/jeremyje/filetools/blob/master/LICENSE)
[![GitHub release](https://img.shields.io/github/release-pre/jeremyje/filetools.svg)](https://github.com/jeremyje/filetools/releases)
[![Build Status](https://travis-ci.org/jeremyje/filetools.svg?branch=master)](https://travis-ci.org/jeremyje/filetools)
[![codecov](https://codecov.io/gh/jeremyje/filetools/branch/master/graph/badge.svg)](https://codecov.io/gh/jeremyje/filetools)

A collection of tools to manage a large collection of files.

 * Find Duplicate files
 * Find similarly named files

Coming Soon

 * Binary releases for Windows, Linux, and MacOs

Unique Bugs

 * Improve Report
  * Sort Items by size, descending
  * File Size Function
 * Actually support different hash algorithms.

Similar
 * Use sharded multiwalk and delete hold multiwalk function since it's prone to race conditions.
 * similar tests are dead locking.
 * Similar cannot handle multiple paths yet because of race-condition in multiwalk acting on 1 map.
