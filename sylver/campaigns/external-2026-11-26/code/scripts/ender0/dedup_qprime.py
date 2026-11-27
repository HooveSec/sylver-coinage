#!/usr/bin/env python3
# Q' minus duplicates: drop extra keys already in Q (checkq sizes its table by record count, 2^27 would exceed 2.5 GB). Sylver worker 07:45 EDT.
import numpy as np
Q='/workspace/sylver/fin3/j_38_68_72_257.bin'; P='qprime_16_38_68_72_257.bin'
hdr=open(Q,'rb').read(64)
q=np.fromfile(Q,dtype=np.uint8,offset=64).reshape(-1,16); p=np.fromfile(P,dtype=np.uint8,offset=64).reshape(-1,16)
ex=p[len(q):]; qv=np.ascontiguousarray(q[:,:15]).view('V15').ravel(); ev=np.ascontiguousarray(ex[:,:15]).view('V15').ravel()
keep=ex[~np.isin(ev,qv)]
with open(P,'wb') as f: f.write(hdr); f.write(q.tobytes()); f.write(keep.tobytes())
print(P,'Q',len(q),'new extra keys',len(keep))
