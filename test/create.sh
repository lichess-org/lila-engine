#!/bin/sh
http -A bearer -a $PAT POST http://localhost:9663/api/external-engine engineName="Stockfish 15" maxThreads=1 maxHashMib=16 secret=reallyReallySecret
