#!/bin/sh
# Run from project root and output script to generate symlink
set -e
version=`./bin/version.sh`
major_version=`./bin/major_version.sh`

(cd deb_assets &&
ln -sf libcskk.so.$version libcskk.so.$major_version &&
ln -sf libcskk.so.$version libcskk.so)
