#!/bin/sh -e
while true; do
    echo Waiting ...
    job=$(http --print b POST http://localhost:9666/api/external-engine/work providerSecret=reallyReallySecret)
    if [ -n "$job" ]; then
        job_id=$(echo "$job" | jq -r .id)
        echo "Now analysing $job_id ..."
        stockfish go depth 20 | http --chunked POST "http://localhost:9666/api/external-engine/work/$job_id"
    fi
done
