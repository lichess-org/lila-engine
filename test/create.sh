#!/bin/sh
http -A bearer -a $PAT POST http://localhost:9663/api/external-engine name="Stockfish 15" maxThreads=1 maxHash=16 providerSecret=reallyReallySecret
