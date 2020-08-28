#!/bin/bash

preload () {
    local library
    library=$1
    shift
    if [ "$(uname)" = "Darwin" ]; then
        REDHOOK_TRACE=/tmp/wisk_trace.log DYLD_INSERT_LIBRARIES=target/debug/"$library".dylib "$@"
    else
        REDHOOK_TRACE=/tmp/wisk_trace.log LD_PRELOAD=target/debug/"$library".so "$@"
        # LD_PRELOAD=target/debug/"$library".so "$@"
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
preload libvarprintspy ./testprog || exit

preload libvarprintspy ./testprog | grep "^readlink('/tmp/wisk_testlink') -> Intercepted" || exit

preload libvarprintspy ./testprog | grep "^Rust: vprintf('Hello World! from vprintf: %d %f %s" || exit
preload libvarprintspy ./testprog | grep "^Hello World! from vprintf: 100 1.234560 something" || exit

preload libvarprintspy ./testprog | grep "^Rust: dprintf('Hello World! from printf: %d %f %s" || exit
preload libvarprintspy ./testprog | grep "^Rust: vprintf('Hello World! from printf: %d %f %s" || exit
preload libvarprintspy ./testprog | grep "^Hello World! from printf: 100 1.234560 something" || exit
test -f /tmp/wisk_trace.log && cat /tmp/wisk_trace.log
# printf "\n\nC LD_PRELOAD"
# cc -fPIC --shared -o target/debug/libtestprog.so src/libtest.c
# preload libtestprog ./testprog
popd

pushd examples/readlinkspy
cargo update
cargo build
preload libreadlinkspy ls -l /dev/stdin | grep readlink
test -f /tmp/wisk_trace.log && cat /tmp/wisk_trace.log
popd

# cd ../neverfree
# cargo update
# cargo build
