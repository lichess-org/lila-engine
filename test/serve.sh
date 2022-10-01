#!/bin/sh -e
while true; do
    echo "# Waiting ..."
    job=$(http --print b POST http://localhost:9666/api/external-engine/work providerSecret=reallyReallySecret)
    if [ -n "$job" ]; then
        job_id=$(echo "$job" | jq -r .id)
        depth=$(echo "$job" | jq -r .work.maxDepth)
        initial_fen=$(echo "$job" | jq -r .work.initialFen)
        moves=$(echo "$job" | jq -r '.work.moves | join(" ")')
        command="position fen $initial_fen moves $moves\ngo depth $depth"
        echo -e "$command"
        cat <(echo -e "$command") - | stockfish | http --chunked POST "http://localhost:9666/api/external-engine/work/$job_id"
    fi
done
