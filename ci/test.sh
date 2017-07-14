#!/usr/bin/env sh

set -e -u

ln -fs $PWD/cargo $HOME/.cargo
ln -fs $PWD/maven $HOME/.m2

PATH=/usr/local/bin:$PATH

cd jvmkill
/usr/local/bin/cargo test --color=always --all -- --test-threads=1 --nocapture
