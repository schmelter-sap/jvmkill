#!/usr/bin/env sh

set -e

export JFROG_CLI_OFFER_CONFIG=false

cd jvmkill
make
/usr/local/bin/jfrog rt upload \
  --url https://repo.spring.io \
  --user $ARTIFACTORY_USERNAME \
  --password $ARTIFACTORY_PASSWORD \
  libjvmkill.so \
  $ARTIFACTORY_REPOSITORY/org/cloudfoundry/jvmkill/$VERSION/libjvmkill-$(echo $VERSION | sed "s|SNAPSHOT|$(date '+%Y%m%d.%H%M%S')|")-$PLATFORM.so
