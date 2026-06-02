#!/bin/bash

echo "Starting indexer with GIT_SHA: $GIT_SHA"

if [[ ! "$GIT_SHA" =~ ^[0-9a-fA-F]{10,64}$ ]]; then
  echo "GIT_SHA must be a hex commit SHA, 10 to 64 characters"
  exit 1
fi

# Schema name starts with git_ and 10 characters of the GIT_SHA
export DATABASE_SCHEMA="git_${GIT_SHA:0:10}"

exec npm run start -- --views-schema=public
