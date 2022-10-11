#!/bin/sh
http -A bearer -a $PAT POST http://localhost:9663/api/external-engine name="Stockfish 15" maxThreads=12 maxHash=4096 shallowDepth=25 deepDepth=25 providerSecret=reallyReallySecret
