#!/usr/bin/env python3
"""Stream fin3 journal -> sorted-unique 15-B key body + 64-B header + sha256 sidecar.
Uses an on-disk sort (chunked) so peak RAM stays ~200 MB. Sylver worker 2026-10-19."""
import hashlib, os, sys, tempfile
J, OUT, CHUNK = sys.argv[1], sys.argv[2], int(sys.argv[3]) if len(sys.argv)>3 else 2_000_000
hdr = open(J,'rb').read(64)
f = open(J,'rb'); f.seek(64)
tmpdir = tempfile.mkdtemp(prefix='qsort_', dir='/workspace/sylver/certq/q100')
parts, buf, n = [], [], 0
def flush():
    global buf, n, parts
    if not buf: return
    buf.sort(); path = f'{tmpdir}/p{len(parts):04d}'
    with open(path,'wb') as o:
        prev=None
        for k in buf:
            if k!=prev: o.write(k); prev=k
    parts.append(path); buf=[]; n=0
while True:
    rec = f.read(16)
    if len(rec)<16: break
    buf.append(rec[:15]); n+=1
    if n>=CHUNK: flush()
flush(); f.close()
# k-way merge
fps = [open(p,'rb') for p in parts]
heads = []
for i,fp in enumerate(fps):
    k=fp.read(15)
    if k: heads.append((k,i))
heads.sort()
h = hashlib.sha256(); h.update(hdr); count=0; prev=None
with open(OUT,'wb') as o:
    o.write(hdr)
    while heads:
        k,i = heads.pop(0)
        if k!=prev:
            o.write(k+b'\0'); h.update(k); count+=1; prev=k  # 16-B record (key+pad) to match checkq's 16-B stride
        nxt=fps[i].read(15)
        if nxt:
            # insert sorted
            lo,hi=0,len(heads)
            while lo<hi:
                m=(lo+hi)//2
                if heads[m][0]<nxt: lo=m+1
                else: hi=m
            heads.insert(lo,(nxt,i))
for fp in fps: fp.close()
import shutil; shutil.rmtree(tmpdir, ignore_errors=True)
open(OUT+'.sha256','w').write(h.hexdigest()+f'  count={count}\n')
print(OUT, 'unique', count, 'sha256', h.hexdigest()[:16])
