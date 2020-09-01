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
cc -Werror -o testprog src/test.c || exit "Testprog Compile Error"
rm -f /tmp/wisk_trace.log
touch /tmp/wisk_testfile
ln -sf /tmp/wisk_testfile /tmp/wisk_testlink
printf "\n\nRUST LD_PRELOAD"
# preload libvarprintspy ./testprog printf || exit
preload libvarprintspy ./testprog readlink || exit

preload libvarprintspy ./testprog readlink | grep "^readlink(/tmp/wisk_testlink)" || exit

preload libvarprintspy ./testprog vprintf || exit
preload libvarprintspy ./testprog vprintf | grep "^Hello World! from vprintf: 100 1.234560 something" || exit

preload libvarprintspy ./testprog printf || exit
preload libvarprintspy ./testprog printf | grep "^Hello World! from printf: 100 1.234560 something" || exit

preload libvarprintspy ./testprog creat-cw || exit
preload libvarprintspy ./testprog creat-cw | grep "^open(/tmp/created.file,65(CREAT),0)" || exit
preload libvarprintspy ./testprog creat-r || exit
preload libvarprintspy ./testprog creat-r | grep "^open(/tmp/created.file,0)" || exit

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
