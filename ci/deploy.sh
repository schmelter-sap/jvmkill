#!/usr/bin/env bash

set -e -u

[[ -d $PWD/cargo && ! -d $HOME/.cargo ]] && ln -s $PWD/cargo $HOME/.cargo
[[ -d $PWD/maven && ! -d $HOME/.m2 ]] && ln -s $PWD/maven $HOME/.m2

PATH=/usr/local/bin:$PATH

/usr/local/bin/cargo --version

export CARGO_UNSTABLE_SPARSE_REGISTRY=true

cd jvmkill

VERSION=$(cargo metadata --format-version=1 --no-deps | jq '.workspace_members[] | select(. | startswith("jvmkill "))' | cut -d ' ' -f 2 |  sed 's|-|.|')
if [ -z "${VERSION:-}" ]; then
  echo "Version [${VERSION:-}] is empty, but required"
  exit 255
fi
echo "Building version $VERSION"

/usr/local/bin/cargo build --color=always --release -p jvmkill

JFROG_CLI_OFFER_CONFIG=false /usr/local/bin/jfrog rt upload \
  --url https://repo.spring.io \
  --user $ARTIFACTORY_USERNAME \
  --password $ARTIFACTORY_PASSWORD \
  $(ls target/release/libjvmkill.* | grep 'dylib\|so') \
  $ARTIFACTORY_REPOSITORY/org/cloudfoundry/jvmkill/$VERSION/jvmkill-$(echo $VERSION | sed "s|SNAPSHOT|$(date '+%Y%m%d.%H%M%S')|").so
