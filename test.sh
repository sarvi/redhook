#!/bin/bash

preload () {
    local library
    library=$1
    shift
    if [ "$(uname)" = "Darwin" ]; then
        DYLD_INSERT_LIBRARIES=target/debug/"$library".dylib "$@"
    else
        LD_PRELOAD=target/debug/"$library".so "$@"
    fi
}

set -ex
set -o pipefail

pushd examples/varprintspy
cargo clean
cargo update
cargo build
gcc -o testprog src/test.c
preload libvarprintspy ./testprog | grep "^vprintf" || exit
preload libvarprintspy ./testprog | grep "^printf" || exit
popd

pushd examples/readlinkspy
cargo update
cargo build
preload libreadlinkspy ls -l /dev/stdin | grep readlink
popd

# cd ../neverfree
# cargo update
# cargo build
