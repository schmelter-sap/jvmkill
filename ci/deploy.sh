#!/usr/bin/env bash

set -e -u

[[ -d $PWD/cargo && ! -d $HOME/.cargo ]] && ln -s $PWD/cargo $HOME/.cargo
[[ -d $PWD/maven && ! -d $HOME/.m2 ]] && ln -s $PWD/maven $HOME/.m2

PATH=/usr/local/bin:$PATH

/usr/local/bin/cargo --version

cd jvmkill
/usr/local/bin/cargo build --color=always --release -p jvmkill

LIBRARY=target/release/libjvmkill.{dylib,so}

JFROG_CLI_OFFER_CONFIG=false /usr/local/bin/jfrog rt upload \
  --url https://repo.spring.io \
  --user $ARTIFACTORY_USERNAME \
  --password $ARTIFACTORY_PASSWORD \
  $LIBRARY \
  $ARTIFACTORY_REPOSITORY/org/cloudfoundry/jvmkill/$VERSION/jvmkill-$(echo $VERSION | sed "s|SNAPSHOT|$(date '+%Y%m%d.%H%M%S')|")-$PLATFORM.so
