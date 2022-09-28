#!/bin/sh -e
while true; do
    echo Waiting ...
    http --print b POST http://localhost:9666/api/external-engine/work providerSecret=reallyReallySecret > /tmp/job
    job_id=$(cat /tmp/job | jq -r .id || echo "")
    if [ -n "$job_id" ]; then
        echo Now analysing ...
        stockfish go depth 20 | http --chunked POST "http://localhost:9666/api/external-engine/work/$job_id"
    fi
done
