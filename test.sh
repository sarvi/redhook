#!/bin/bash

preload () {
    local library
    library=$1
    shift
    if [ "$(uname)" = "Darwin" ]; then
        WISK_TRACEFILE=/tmp/wisk_trace.log DYLD_INSERT_LIBRARIES=target/debug/"$library".dylib "$@"
    else
        WISK_TRACEFILE=/tmp/wisk_trace.log LD_PRELOAD=target/debug/"$library".so "$@"
    fi
}

set -ex
set -o pipefail

pushd examples/varprintspy
# cargo clean
# cargo update
cargo build
cc -o testprog src/test.c
rm -f /tmp/wisk_trace.log
touch /tmp/wisk_testfile
ln -sf /tmp/wisk_testfile /tmp/wisk_testlink
printf "\n\nRUST LD_PRELOAD"
preload libvarprintspy ./testprog | grep "^readlink('/tmp/wisk_testlink') -> Intercepted" || exit
preload libvarprintspy ./testprog | grep "^Rust: vprintf('Hello World! from vprintf') -> Intercepted" || exit
preload libvarprintspy ./testprog | grep "^Rust: dprintf('Hello World! from printf') -> Intercepted" || exit
preload libvarprintspy ./testprog | grep "^Rust: vprintf('Hello World! from printf') -> Intercepted" || exit
cat /tmp/wisk_trace.log
# printf "\n\nC LD_PRELOAD"
# cc -fPIC --shared -o target/debug/libtestprog.so src/libtest.c
# preload libtestprog ./testprog
popd

pushd examples/readlinkspy
cargo update
cargo build
preload libreadlinkspy ls -l /dev/stdin | grep readlink
popd

# cd ../neverfree
# cargo update
# cargo build
