#!/bin/sh
if [ $# -eq 1 ]; then
	git cliff -o CHANGELOG.md --tag "$1" && git add CHANGELOG.md
else
	echo "This script should only be called by cargo-release."
	echo "Usage: $0 <version>"
	exit 42
fi
