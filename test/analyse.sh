#!/bin/sh
http --stream POST https://engine.lichess.ovh/api/external-engine/eei_iB8ZF5UtlhC4/analyse clientSecret=ees_2YwuAk2CuO6ERL8a work[sessionId]=hi work[threads]:=1 work[hashMib]:=16 work[maxDepth]:=22 work[multiPv]:=5 work[variant]=standard work[initialFen]="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" work[moves]:=[]
