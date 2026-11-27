import subprocess,sys,os,re,json
from math import gcd
from functools import reduce
from concurrent.futures import ThreadPoolExecutor
TO=int(os.environ.get('XTO','30'))
def run(args,lem=True):
    env=dict(os.environ,VMB='250'); 
    if lem: env['VLEM']='lemmas_all.tsv'
    try: o=subprocess.run(['./vchk',*args],capture_output=True,text=True,timeout=TO,env=env)
    except subprocess.TimeoutExpired: return ('TIMEOUT','')
    m=re.search(r' (P|N|U) move=(-?\d+).*?(lemmas_used=(\S+))?$',o.stdout.strip())
    if not m: return ('INFEASIBLE',o.stderr.strip()[:40])
    return (m.group(1),m.group(2)+(' lem='+m.group(4) if m.group(4) and m.group(4)!='none' else ''))
def evalpos(pos,lem=True):
    if reduce(gcd,pos)==1: return run(['-f',*map(str,pos)],False)
    if reduce(gcd,pos)==2: return run(list(map(str,pos)),lem)
    return ('SKIP-g>2','')
def check(row):
    pos,val,mv,src=row
    p=[int(x) for x in pos.split(',')]
    r=evalpos(p)
    res={'pos':pos,'lit':val,'litmv':mv,'src':src,'vchk':r[0],'vinfo':r[1]}
    if val=='N' and mv:
        m=int(mv.split(',')[0]); c=evalpos(sorted(set(p+[m])))
        res['child']=c[0]; res['childinfo']=c[1]
    return res
rows=[l.rstrip('\n').split('\t') for l in open(sys.argv[1]) if not l.startswith('position')]
rows=[(r+['',''])[:4] if len(r)<4 else r for r in rows]
if len(rows[0])==3 or sys.argv[1].endswith('known.tsv'): rows=[(r[0],r[1],'',r[2] if len(r)>2 else '') for r in rows]
rows=[r for r in rows if 'n' not in r[0]]
with ThreadPoolExecutor(int(os.environ.get('XJ','3'))) as ex, open(sys.argv[2],'w') as f:
    for res in ex.map(check,rows): f.write(json.dumps(res)+'\n'); f.flush()
