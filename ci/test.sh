#!/bin/bash

set -ex

# Change to our project home.
script_dir=`dirname "${BASH_SOURCE[0]}"`
cd "$script_dir"/..

# Print our cargo version, for debugging.
cargo --version

# Force default tests to disable default feature on NO_STD.
if [ ! -z $NO_STD ]; then
    DEFAULT_FEATURES="--no-default-features"
    DOCTESTS="--tests"
fi

# Test various feature combinations.
FEATURES=(
    "compact"
    "alloc"
    "compact,alloc"
)

check() {
    if [ ! -z $NO_FEATURES ]; then
        return
    fi

    # Need to test a few permutations just to ensure everything compiles.
    for features in "${FEATURES[@]}"; do
        check_features="$DEFAULT_FEATURES --features=$features"
        cargo check --tests $check_features
    done
}

# Build target.
build() {
    cargo build $DEFAULT_FEATURES
    cargo build $DEFAULT_FEATURES --release
}

# Test target.
test() {
    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi
    if [ ! -z $NO_STD ]; then
        return
    fi

    # Default tests.
    cargo test $DEFAULT_FEATURES $DOCTESTS
    cargo test $DEFAULT_FEATURES $DOCTESTS --release
}

# Test target.
test() {
    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    # Default tests.
    cargo test $DEFAULT_FEATURES $DOCTESTS
    cargo test $DEFAULT_FEATURES $DOCTESTS --release
}

main() {
    check
    build
    test

    if [ ! -z $NIGHTLY ]; then
        scripts/check.sh
        RUSTFLAGS="--deny warnings" cargo +nightly build --features=lint
    fi
}
