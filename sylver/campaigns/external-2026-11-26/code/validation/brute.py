import sys,functools,itertools
def rep(gens,lim):
    r=[False]*lim; r[0]=True
    for g in gens:
        for k in range(g,lim):
            if r[k-g]: r[k]=True
    return r
@functools.lru_cache(None)
def win(gens):
    lim=max(gens)**2+5; r=rep(gens,lim)
    gaps=[x for x in range(2,lim) if not r[x]]
    return any(not win(tuple(sorted(gens+(x,)))) for x in gaps)
import random,subprocess
random.seed(1); bad=0
for trial in range(40):
    k=random.choice([2,3]); 
    while True:
        g=tuple(sorted(random.sample(range(3,10),k)))
        from math import gcd; from functools import reduce
        if reduce(gcd,g)==1: break
    w=win(g); o=subprocess.run(['./vchk','-f',*map(str,g)],capture_output=True,text=True,env={'VMB':'50'}).stdout
    c=' N ' in o
    if c!=w: bad+=1; print("MISMATCH",g,w,o)
print("trials 40 mismatches",bad)
