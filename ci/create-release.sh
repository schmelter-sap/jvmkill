#!/usr/bin/env bash

set -e -u

RELEASE=$1
SNAPSHOT=$2

update_release() {
  local file=$1
  local version=$2

  sed -E -i '' "s|(^version = \").*(\".*)$|\1$version\2|" $file
}

update_release jvmkill/Cargo.toml $RELEASE
git add .
git commit --message "v$RELEASE Release"

git tag -s v$RELEASE -m "v$RELEASE"
git reset --hard HEAD^1

update_release jvmkill/Cargo.toml $SNAPSHOT
git add .
git commit --message "v$SNAPSHOT Development"
