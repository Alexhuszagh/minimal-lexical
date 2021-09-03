#!/bin/bash
# Run a small subset of our comprehensive test suite.

set -ex

# Print our cargo version, for debugging.
cargo --version

# Change to our project home.
script_dir=`dirname "${BASH_SOURCE[0]}"`
script_home=`realpath "$script_dir"`
cd "$script_home"/..

FEATURES=
if [ ! -z $ALL_FEATURES ]; then
    FEATURES=--all-features
fi

# Test the parse-float correctness tests
cd etc/correctness
cargo run $FEATURES --release --bin test-parse-golang
cargo run $FEATURES --release --bin test-parse-unittests
