# Frontier map for {16,r}: QET class, odd candidates, and even answers a that make {16,r,a} SHORT (certifiable by QET).
import math, json, sys
def gaps(gens, lim=None):
    g=0
    for x in gens: g=math.gcd(g,x)
    assert g==1
    m=min(gens); lim=lim or (max(gens)*m+5)
    rep=[False]*(lim); rep[0]=True
    for k in range(1,lim):
        rep[k]=any(k>=x and rep[k-x] for x in gens)
    G=[k for k in range(1,lim) if not rep[k]]; return G
def quiet(gens):
    G=gaps(gens); t=max(G) if G else 0; s=set(G)
    if t%2==0: return False,t,G
    return all(((m in s)!=((t-m) in s)) for m in range(1,t)),t,G
def cls(r):
    d=math.gcd(16,r); S=[16//d, r//d]
    q,t,G=quiet(S)
    oddc=[z for z in G if z>1 and z%2==1 and math.gcd(z,d)==1]
    return d,S,q,t,G,oddc
rows=[]
for r in range(38,122,2):
    if r%16==0: continue
    d,S,q,t,G,oddc=cls(r)
    row=dict(r=r,g=d,half=S,quiet=q,t_half=t,genus=len(G),odd_candidates=len(oddc))
    # even answers a (a ≡ 2 mod 4 if d>=4 gives g=2 child) that make {16,r,a} short: half {8,r/2,a/2} (if g=2) quiet ender
    short=[]
    for a in range(2,4*r,2):
        P=[16,r,a]; gg=math.gcd(math.gcd(16,r),a)
        if gg!=2: continue
        H=[x//2 for x in P]
        # canonical: legal?
        try: qq,tt,GG=quiet(H)
        except AssertionError: continue
        # a must be legal in {16,r}: not representable
        if any((a-16*i)>=0 and (a-16*i)%r==0 for i in range(a//16+1)): continue
        if qq: short.append((a,len(GG)))
    short.sort(key=lambda x:x[1])
    row['short_even_children']=len(short); row['cheapest_short']=short[:6]
    rows.append(row)
for w in rows: print(json.dumps(w))
