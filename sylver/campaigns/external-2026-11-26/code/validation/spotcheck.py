#!/usr/bin/env python3
# Spot-check random records of an even2/even3 store file against the INDEPENDENT finite solver ../solve
# (gap-bitset negamax, separate code). Read-only on the store. usage: spotcheck.py STORE NWANT [FMAX] [SEED]
import sys,os,random,subprocess
M64=(1<<64)-1
def hk(k):
    a=bytes(k)+b'\0'; x=int.from_bytes(a[:8],'little'); y=int.from_bytes(a[8:],'little')
    rot=((y<<29)|(y>>35))&M64
    h=((x*0x9E3779B97F4A7C15)&M64)^((rot*0xC2B2AE3D27D4EB4F)&M64)
    h^=h>>31; h=(h*0xFF51AFD7ED558CCD)&M64; h^=h>>33; h=(h*0xC4CEB9FE1A85EC53)&M64; return h^(h>>29)
def chk(h,v): return 0x80|((h>>40)&0x7e)|v
path=sys.argv[1]; want=int(sys.argv[2]); fmax=int(sys.argv[3]) if len(sys.argv)>3 else 110
random.seed(int(sys.argv[4]) if len(sys.argv)>4 else 20261018)
slots=os.path.getsize(path)//16; fd=os.open(path,os.O_RDONLY)
ok=bad=to=badchk=seen=0; tried=0
while ok+bad<want and tried<5_000_000:
    tried+=1; s=random.randrange(slots); rec=os.pread(fd,16,s*16)
    if rec[15]==0: continue
    k=rec[:15]; v=rec[15]&1
    if chk(hk(k),v)!=rec[15]: badchk+=1; continue
    if k[14]>=0xF0: continue   # M<16 record: skip (not used by M=16 jobs)
    w=[r+16*k[r-1] for r in range(1,16)]; f=max(w)-16; seen+=1
    if f>fmax: continue
    try: out=subprocess.run(['bash','-c','ulimit -v 2000000; exec ../solve 16 '+' '.join(map(str,w))],capture_output=True,text=True,timeout=60).stdout
    except subprocess.TimeoutExpired: to+=1; continue
    ind='N' if ' N nodes' in out else ('P' if ' P nodes' in out else '?')
    st='N' if v else 'P'
    if ind==st: ok+=1
    else: bad+=1; print('MISMATCH slot',s,'apery',w,'store',st,'solve',ind,out.strip(),flush=True)
print(f'spotcheck {path}: agree={ok} MISMATCH={bad} timeout={to} badchk={badchk} records_seen={seen} (F<={fmax} checked) slots_probed={tried}')
