#!/bin/bash

set -ex

# Detect our build command. If we enabled cross, default to
# that. Otherwise, only use cross if we are on CI and did
# not explicitly disable it.
if [ ! -z $ENABLE_CROSS ]; then
    # Specifically enabled cross.
    CARGO=cross
    CARGO_TARGET="--target $TARGET"
elif [ -z $CI ] || [ ! -z $DISABLE_CROSS ]; then
    # Explicitly disabled cross, use cargo.
    CARGO=cargo
else
    # On CI, did not disable cross, use cross.
    CARGO=cross
    CARGO_TARGET="--target $TARGET"
fi

# Force default tests to disable default feature on NO_STD.
if [ ! -z $NO_STD ]; then
    DEFAULT_FEATURES="--no-default-features"
    DOCTESTS="--tests"
fi

# Disable doctests on nostd or if not supported.
if [ ! -z $DISABLE_DOCTESTS ]; then
    DOCTESTS="--tests"
fi

# Build target.
build() {
    $CARGO build $CARGO_TARGET $DEFAULT_FEATURES
    $CARGO build $CARGO_TARGET $DEFAULT_FEATURES --release
    if [ -z $DISABLE_BINARIES ]; then
        $CARGO build $CARGO_TARGET $DEFAULT_FEATURES --features=$no_alloc,rng,examples,comprehensive_float_test
    fi
}

# Test target.
test() {
    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    # Default tests.
    $CARGO test $CARGO_TARGET $DEFAULT_FEATURES $DOCTESTS
    $CARGO test $CARGO_TARGET $DEFAULT_FEATURES $DOCTESTS --release
}

main() {
    build
    test
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
