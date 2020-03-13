#!/bin/sh

VERSION=$1

echo "$2" | base64 -d > /tmp/credentials.json
gcloud auth activate-service-account --key-file=/tmp/credentials.json

gsutil rsync -r -d std gs://cdn.loalang.xyz/$VERSION/std
tree -J std | jq '.[0].contents' | gsutil cp - gs://cdn.loalang.xyz/$VERSION/std/manifest.json
gsutil setmeta -h "Content-Type: application/loa" gs://cdn.loalang.xyz/$VERSION/std/**/*.loa

