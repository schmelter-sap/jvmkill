#!/usr/bin/env sh

set -e -u

[[ -d $PWD/cargo && ! -d $HOME/.cargo ]] && ln -s $PWD/cargo $HOME/.cargo
[[ -d $PWD/maven && ! -d $HOME/.m2 ]] && ln -s $PWD/maven $HOME/.m2

PATH=/usr/local/bin:$PATH

cd jvmkill
/usr/local/bin/cargo test --color=always --all -- --test-threads=1 --nocapture
