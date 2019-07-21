#!/bin/sh

FEAT_TR="${1}"
[ -n "$FEAT_TR" ] && FEAT_TR="--features '$FEAT_TR'"

CARGO_FLAGS="--verbose $FEAT_TR"

eval cargo build $CARGO_FLAGS || exit $?
eval cargo test $CARGO_FLAGS || exit $?

echo
echo "=== run clippy ..."
if rustup component add clippy-preview; then
    eval cargo clippy $CARGO_FLAGS
else
    echo -e '\033[1m\033[33mWARNING\033[0m: clippy not found'
fi
