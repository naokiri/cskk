#!/bin/sh
# Run from project root and output script to generate symlink
set -e
version=`./deploy_scripts/version.sh`
major_version=`./deploy_scripts/major_version.sh`

(cd cskk/deb_assets &&
ln -sf libcskk.so.$version libcskk.so &&
ln -sf libcskk.so.$version libcskk.so.$major_version &&
ln -sf libcskk.so.$major_version libcskk.so)
