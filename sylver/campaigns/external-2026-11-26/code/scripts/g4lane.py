#!/usr/bin/env python3
# g4 lane v2 (2026-10-18, Math Lab Sylver worker). Position T (multiples of 4, e.g. 16 28). Looks for a reply x with
# T+[x] P (then T is N, rigorous). Even x≡2 mod 4 solved by even3 (g=2 solver; finite-game values persisted in a private
# store, optional READ-ONLY base layer = another job's store). Smallest reply first, long per-try limit (TRY_TO s),
# graceful DEADLINE inside even3 (store flushed, TIMEOUT never cached). Resumable: P/N go to ../g2_cache.tsv (shared with
# g4.py); tries.tsv records every attempt (TIMEOUT rows only schedule retries: a reply is retried when TRY_TO grows).
import subprocess, sys, os, json, time, re, signal
from math import gcd
from functools import reduce
D='/workspace/sylver/g4lane'; os.makedirs(D+'/logs',exist_ok=True); CACHE='/workspace/sylver/g2_cache.tsv'; TRIES=D+'/tries.tsv'
E=int(os.environ.get('EMAX','130')); TO=int(os.environ.get('TRY_TO','10800'))
EV3=os.environ.get('EVEN3','/workspace/sylver/even3/even3')
ENV=dict(ODDB='300',PLIM='60001',PULIM=os.environ.get('PULIM','1500000'),STORE=os.environ.get('G4STORE',D+'/g4store27.bin'),BITS='27',
         RAMIDX=os.environ.get('G4RAMIDX','26'),FLUSH='200000',CACHE='3000000',DMIN='8',ETC='1',KILLER='1',RATE_SEC='600')
if os.environ.get('G4BASE','')!='': ENV.update(BASE=os.environ['G4BASE'],BASE_BITS=os.environ.get('G4BASE_BITS','29'))
def in_semi(gens,x):
    r=[False]*(x+1); r[0]=True
    for n in range(1,x+1): r[n]=any(n>=g and r[n-g] for g in gens)
    return r[x]
def mingens(v):
    v=sorted(set(v)); out=[]
    for x in v:
        if not out or not in_semi(out,x): out.append(x)
    return out
def cache_get(key):
    try:
        for l in open(CACHE):
            k,r=l.rstrip('\n').split('\t')[:2]
            if k==key: return r
    except FileNotFoundError: pass
    return None
def tried_to(T,x):
    best=0
    try:
        for l in open(TRIES):
            f=l.rstrip('\n').split('\t')
            if f[0]==','.join(map(str,T)) and int(f[1])==x and f[2]=='TIMEOUT': best=max(best,int(f[4]))
    except FileNotFoundError: pass
    return best
def log(msg): open('/workspace/sylver/progress.log','a').write(time.strftime('%F %T %Z')+' g4lane '+msg+'\n')
child=None
def term(sig,frm):
    if child is not None:
        try: os.killpg(child.pid,signal.SIGTERM)
        except Exception: pass
    sys.exit(143)
signal.signal(signal.SIGTERM,term)
def run_even3(gens,x):
    global child
    e=dict(os.environ,**ENV,DEADLINE=str(TO)); errf=open(f'{D}/logs/{"_".join(map(str,gens))}.err','a')
    errf.write(time.strftime('%F %T %Z')+f' start TRY_TO={TO} env={json.dumps(ENV)}\n'); errf.flush(); t=time.time()
    child=subprocess.Popen(['bash','-c','ulimit -v 2950000; exec '+EV3+' '+' '.join(map(str,gens))],stdout=subprocess.PIPE,stderr=errf,text=True,env=e,start_new_session=True)
    try: out,_=child.communicate(timeout=TO+1800)
    except subprocess.TimeoutExpired:
        os.killpg(child.pid,signal.SIGKILL); child.communicate(); out='HARDTIMEOUT'
    rc=child.returncode; child=None; secs=round(time.time()-t)
    m=re.search(r' (P|N|UNKNOWN) nodes=(\d+).* pfail=(\d+)',out)
    if m: return m.group(1),secs,int(m.group(3)),out.strip()
    if 'TIMEOUT' in out: return 'TIMEOUT',secs,-1,out.strip()
    return f'ERR(rc={rc})',secs,-1,out.strip()
def run_fin3(gens,x):
    global child
    tag='_'.join(map(str,gens)); e=dict(os.environ,PBITS='26',NBITS='22',ENDER='1',RATE_SEC='600',PJOURNAL=f'{D}/logs/j_{tag}.bin')
    errf=open(f'{D}/logs/{tag}.err','a'); t=time.time()
    child=subprocess.Popen(['bash','-c','ulimit -v 2000000; exec /workspace/sylver/fin3/fin3 '+' '.join(map(str,gens))],stdout=subprocess.PIPE,stderr=errf,text=True,env=e,start_new_session=True)
    try: out,_=child.communicate(timeout=TO)
    except subprocess.TimeoutExpired:
        os.killpg(child.pid,signal.SIGTERM); child.communicate(); out='TIMEOUT'
    rc=child.returncode; child=None; secs=round(time.time()-t)
    m=re.search(r' (P|N) nodes=',out)
    if m: return m.group(1),secs,0,out.strip()
    return ('TIMEOUT' if out=='TIMEOUT' else f'ERR(rc={rc})'),secs,-1,out.strip()
def main():
    T=list(map(int,sys.argv[1:])); key=','.join(map(str,T)); log(f'start T={T} TRY_TO={TO} EMAX={E} env={ENV}')
    odds=[int(a) for a in os.environ.get('ODDS','').split(',') if a]
    for x in odds:   # finite odd candidates (fin3, P-journal resumes), checked before the even sweep
        g=sorted(T+[x]); ck='g1:'+','.join(map(str,mingens(g))); r=cache_get(ck)
        if r is None and tried_to(T,x)>=TO: continue
        if r is None:
            r,secs,pf,out=run_fin3(g,x)
            open(TRIES,'a').write(f'{key}\t{x}\t{r}\t{secs}\t{TO}\tfin3\t{time.strftime("%F %T %Z")}\t{out[:200]}\n')
            log(f'{T} odd reply {x}: {r} in {secs}s (fin3)')
            if r in ('P','N'): open(CACHE,'a').write(f"{ck}\t{r}\tg4lane fin3\n")
        if r=='P': print(json.dumps({'T':T,'value':'N','win':x})); log(f'RESULT {T} is N via odd {x} (child P, fin3)'); return 0
    if os.environ.get('ODD_ONLY')=='1': log(f'odd pass done T={T}'); return 0
    G=reduce(gcd,T); step=2 if G==2 else 4
    for x in range(2,E+1,step):
        if in_semi(T,x): continue
        g=mingens(T+[x]); ck='g2:'+','.join(map(str,g)); r=cache_get(ck)
        if r is None and tried_to(T,x)>=TO: continue
        if r is None:
            r,secs,pf,out=run_even3(T+[x],x)
            open(TRIES,'a').write(f'{key}\t{x}\t{r}\t{secs}\t{TO}\tpfail={pf}\t{time.strftime("%F %T %Z")}\t{out[:200]}\n')
            log(f'{T} reply {x}: {r} in {secs}s (TRY_TO={TO}, pfail={pf})')
            if r in ('P','N'): open(CACHE,'a').write(f"{ck}\t{r}\tg4lane even3 TRY_TO={TO}\n")
        if r=='P': print(json.dumps({'T':T,'value':'N','win':x})); log(f'RESULT {T} is N via {x} (child P)'); return 0
    log(f'pass done T={T}: no P child among even replies <= {E} at TRY_TO={TO}'); print(json.dumps({'T':T,'value':'UNKNOWN'})); return 0
if __name__=='__main__': sys.exit(main())
