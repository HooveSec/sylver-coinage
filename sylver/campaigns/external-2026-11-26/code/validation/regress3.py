#!/usr/bin/env python3
# Regression for even3 in the g4-lane configuration (RAMIDX + ETC + KILLER, scratch store) against literature
# (known.tsv all gcd-2 rows, known_blok all gcd-2 P rows, random sample of gcd-2 N rows). 0 contradictions required.
import subprocess,random,json,os,re
from math import gcd
from functools import reduce
random.seed(int(os.environ.get('SEED','20261018')))
TO=int(os.environ.get('RTO','20'))
def rows(fn,src):
    out=[]
    for l in open(fn):
        if l.startswith('position'): continue
        r=l.rstrip('\n').split('\t')
        if 'n' in r[0]: continue
        p=list(map(int,r[0].split(',')))
        if reduce(gcd,p)!=2 or min(p)>32: continue
        out.append((r[0],r[1],src))
    return out
K=rows('../known.tsv','known.tsv'); B=rows('../known_blok.tsv','known_blok')
BP=[r for r in B if r[1]=='P']; BN=[r for r in B if r[1]=='N']
sample=K+BP+random.sample(BN,min(len(BN),int(os.environ.get('NN','150'))))
def run(pos):
    e=dict(os.environ,ODDB='300',PLIM='100001',PULIM='2500000',STORE='/workspace/sylver/even3/reg3.bin',BITS='26',RAMIDX='25',CACHE='2000000',DMIN='8',ETC='1',KILLER='1',RATE_SEC='100000')
    p=subprocess.Popen(['bash','-c','ulimit -v 3000000; exec taskset -c %s ./even3 %s'%(os.environ.get('CPU','2'),pos.replace(',',' '))],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,env=e,start_new_session=True)
    try: out,err=p.communicate(timeout=TO)
    except subprocess.TimeoutExpired:
        os.killpg(p.pid,15); p.communicate(); return 'TIMEOUT',0
    if p.returncode!=0: return 'FAIL(%d)'%p.returncode,0
    m=re.search(r' (P|N|UNKNOWN) nodes=\d+ .*pfail=(\d+)',out); return (m.group(1),int(m.group(2))) if m else ('PARSE',0)
c={'agree':0,'CONTRA':0,'UNKNOWN':0,'TIMEOUT':0,'other':0}; pf=0
prev={}
if os.path.exists('regress3.jsonl'):
    for l in open('regress3.jsonl'):
        d=json.loads(l); prev[d['pos']+'|'+d['src']]=(d['even3'],d['pfail'])
with open('regress3.jsonl','a') as f:
    for pos,lit,src in sample:
        if pos+'|'+src in prev:
            r,p=prev[pos+'|'+src]; pf+=p
            k='agree' if r==lit else ('CONTRA' if r in ('P','N') else (r if r in c else 'other')); c[k]+=1; continue
        r,p=run(pos); pf+=p
        k='agree' if r==lit else ('CONTRA' if r in ('P','N') else (r if r in c else 'other')); c[k]+=1
        f.write(json.dumps({'pos':pos,'lit':lit,'src':src,'even3':r,'pfail':p})+'\n'); f.flush()
print('regress3:',len(sample),'positions',c,'pfail_total',pf)
