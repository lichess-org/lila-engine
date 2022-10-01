#!/bin/sh -e
while true; do
    echo "# Waiting ..."
    job=$(http --print b POST http://localhost:9666/api/external-engine/work providerSecret=reallyReallySecret)
    if [ -n "$job" ]; then
        job_id=$(echo "$job" | jq -r .id)
        depth=$(echo "$job" | jq -r .work.maxDepth)
        command="go depth $depth"
        echo "$command"
        cat <(echo "$command") - | stockfish | http --chunked --stream POST "http://localhost:9666/api/external-engine/work/$job_id"
    fi
done
