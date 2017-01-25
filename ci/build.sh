#!/usr/bin/env sh

set -e

cd jvmkill
make
JFROG_CLI_OFFER_CONFIG=false /usr/local/bin/jfrog rt upload \
  --url https://repo.spring.io \
  --user $ARTIFACTORY_USERNAME \
  --password $ARTIFACTORY_PASSWORD \
  libjvmkill.so \
  $ARTIFACTORY_REPOSITORY/org/cloudfoundry/jvmkill/$VERSION/jvmkill-$(echo $VERSION | sed "s|SNAPSHOT|$(date '+%Y%m%d.%H%M%S')|")-$PLATFORM.so
