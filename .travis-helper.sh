#!/bin/sh

FEAT_TR="${1}"
[ -n "$FEAT_TR" ] && FEAT_TR="--features '$FEAT_TR'"

CARGO_FLAGS="--verbose $FEAT_TR"

eval cargo build $CARGO_FLAGS || exit $?
eval cargo test $CARGO_FLAGS || exit $?

echo
echo "=== run clippy ..."
rustup component add clippy-preview && eval cargo clippy $CARGO_FLAGS || true
