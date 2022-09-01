#!/bin/sh
# Run from repository root and output verson from Cargo.toml
# Helper for release automation

set -e
grep ^version Cargo.toml | cut -d' ' -f3 | sed -e 's/"//g'
