#!/usr/bin/env python3
# list distinct ender children from an ENDERLOG (15-byte Kunz key mod 16 + pad) as minimal generator sets, one per line, first-seen order
import sys
d=open(sys.argv[1],'rb').read(); M=16; seen=set()
for i in range(len(d)//16):
    k=bytes(d[16*i:16*i+15])
    if k in seen: continue
    seen.add(k); w=[0]+[r+M*k[r-1] for r in range(1,M)]
    inS=lambda n: n>=w[n%M]
    F=max(w)-M; elems=[n for n in range(1,F+M+1) if inS(n)]
    gens=[]
    for n in elems:   # n is a minimal generator iff not a sum of two nonzero elements
        if not any(inS(n-a) for a in elems if 0<a<n and n-a>0 and a<=n-a): gens.append(n)
    print(k.hex(),' '.join(map(str,gens)))
