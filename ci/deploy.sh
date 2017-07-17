#!/usr/bin/env sh

set -e -u

[ -d $PWD/cargo ] && ln -fs $PWD/cargo $HOME/.cargo
[ -d $PWD/maven ] && ln -fs $PWD/maven $HOME/.m2

PATH=/usr/local/bin:$PATH

cd jvmkill
/usr/local/bin/cargo build --color=always --release -p jvmkill

JFROG_CLI_OFFER_CONFIG=false /usr/local/bin/jfrog rt upload \
  --url https://repo.spring.io \
  --user $ARTIFACTORY_USERNAME \
  --password $ARTIFACTORY_PASSWORD \
  target/release/libjvmkill.* \
  $ARTIFACTORY_REPOSITORY/org/cloudfoundry/jvmkill/$VERSION/jvmkill-$(echo $VERSION | sed "s|SNAPSHOT|$(date '+%Y%m%d.%H%M%S')|")-$PLATFORM.so
