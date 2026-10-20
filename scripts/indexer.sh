#!/bin/bash

# Bump to reindex into a fresh schema. Needed when contracts, ponder.schema.ts or src/ change.
SCHEMA_VERSION="fe4e39430f"

echo "Starting indexer with GIT_SHA: $GIT_SHA, schema version: $SCHEMA_VERSION"

export DATABASE_SCHEMA="git_${SCHEMA_VERSION}"

exec npm run start -- --views-schema=public
