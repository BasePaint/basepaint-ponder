#!/bin/bash

# Fail if no GIT_SHA is set
if [ -z "$GIT_SHA" ]; then
  echo "GIT_SHA is not set"
  exit 1
fi

# Schema name starts with git_ and 10 characters of the GIT_SHA
export DATABASE_SCHEMA="git_${GIT_SHA:0:10}"

exec npm run start -- --views-schema=public

