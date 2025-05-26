#!/usr/bin/env bash

set -x
set -eo pipefail

RUNNING_CONTAINER=$(docker ps --filter 'name=minio' --format "{{.ID}}")
if [[ -n $RUNNING_CONTAINER ]]; then
    echo >&2 "there is a minio container already running, kill it with"
    echo >&2 "  docker kill ${RUNNING_CONTAINER}"
    exit 1
fi

docker run \
    -p "9000:9000" \
    -p "9001:9001" \
    -e "MINIO_ROOT_USER=minioadmin" \
    -e "MINIO_ROOT_PASSWORD=minioadmin" \
    -d \
    --name "minio_$(date '+%s')" \
    minio/minio server /data --console-address ":9001"

>&2 echo "MinIO is ready to go!"
echo >&2 "API: http://localhost:9000"
echo >&2 "Console: http://localhost:9001"
echo >&2 "Access Key: minioadmin"
echo >&2 "Secret Key: minioadmin"
