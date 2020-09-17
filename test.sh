#!/bin/bash

preload () {
    local library
    library=$1
    shift
    if [ "$(uname)" = "Darwin" ]; then
        REDHOOK_TRACE=/tmp/wisk_trace.log DYLD_INSERT_LIBRARIES=target/debug/"$library".dylib "$@"
    else
#        REDHOOK_TRACE=/tmp/wisk_trace.log LD_PRELOAD=target/i686-unknown-linux-gnu/debug/"$library".so "$@"
        REDHOOK_TRACE=/tmp/wisk_trace.log LD_PRELOAD=target/debug/"$library".so "$@"
        # LD_PRELOAD=target/debug/"$library".so "$@"
    fi
}

set -ex
set -o pipefail

pushd examples/varprintspy
# cargo clean
# cargo update
# cargo build
# cargo build --target=i686-unknown-linux-gnu
cargo build
cc -Werror -o testprog64 src/test.c || exit "Testprog Compile Error"
# cc -Werror -m32 -o testprog32 src/test.c || exit "Testprog Compile Error"
rm -f /tmp/wisk_trace.log
touch /tmp/wisk_testfile
ln -sf /tmp/wisk_testfile /tmp/wisk_testlink
printf "\n\nRUST LD_PRELOAD"

preload libvarprintspy ./testprog64 readlink || exit
preload libvarprintspy ./testprog64 readlink | grep "^readlink(/tmp/wisk_testlink)" || exit

preload libvarprintspy ./testprog64 vprintf || exit
preload libvarprintspy ./testprog64 vprintf | grep "^Hello World! from vprintf: 100 1.234560 something" || exit

preload libvarprintspy ./testprog64 printf || exit
preload libvarprintspy ./testprog64 printf | grep "^Hello World! from printf: 100 1.234560 something" || exit

preload libvarprintspy ./testprog64 creat-cw || exit
preload libvarprintspy ./testprog64 creat-cw | grep "^open(/tmp/created.file,65(CREAT),0)" || exit
preload libvarprintspy ./testprog64 creat-r || exit
preload libvarprintspy ./testprog64 creat-r | grep "^open(/tmp/created.file,0)" || exit

test -f /tmp/wisk_trace.log && cat /tmp/wisk_trace.log
# printf "\n\nC LD_PRELOAD"
# cc -fPIC --shared -o target/debug/libtestprog.so src/libtest.c
# preload libtestprog ./testprog
popd

pushd examples/readlinkspy
cargo update
cargo build
preload libreadlinkspy ls -l /dev/stdin
preload libreadlinkspy ls -l /dev/stdin | grep readlink
test -f /tmp/wisk_trace.log && cat /tmp/wisk_trace.log
popd

pushd examples/safereadlinkspy
cargo update
cargo build
preload libsafereadlinkspy rustc -vV
preload libsafereadlinkspy rustc -vV | grep "binary: rustc"
test -f /tmp/wisk_trace.log && cat /tmp/wisk_trace.log
popd

# cd ../neverfree
# cargo update
# cargo build
