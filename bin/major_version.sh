#!/bin/sh
# Run from repository root and output major version only
# Helper for release automation.
bin/version.sh | cut -d'.' -f 1