#!/bin/sh

PATCH_TAG=$1
MINOR_TAG=$(echo $PATCH_TAG | sed 's/\.[^.]*$//g')
MAJOR_TAG=$(echo $MINOR_TAG | sed 's/\.[^.]*$//g')

docker login -u $2 -p $3

docker build -t loalang/base:latest -f docker/base.dockerfile .
docker build -t loalang/loa-base:latest -f docker/loa-base.dockerfile .
docker build -t loalang/vm-base:latest -f docker/vm-base.dockerfile .
docker build -t loalang/loa:${PATCH_TAG} \
             -t loalang/loa:${MINOR_TAG} \
             -t loalang/loa:${MAJOR_TAG} \
             -t loalang/loa:latest \
             -f docker/loa.dockerfile .
docker build -t loalang/vm:${PATCH_TAG} \
             -t loalang/vm:${MINOR_TAG} \
             -t loalang/vm:${MAJOR_TAG} \
             -t loalang/vm:latest \
             -f docker/vm.dockerfile .

docker push loalang/loa:${PATCH_TAG}
docker push loalang/loa:${MINOR_TAG}
docker push loalang/loa:${MAJOR_TAG}
docker push loalang/loa:latest
docker push loalang/vm:${PATCH_TAG}
docker push loalang/vm:${MINOR_TAG}
docker push loalang/vm:${MAJOR_TAG}
docker push loalang/vm:latest
