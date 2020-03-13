FROM google/cloud-sdk:284.0.0-alpine

RUN apk update && apk add tree

COPY entrypoint.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
