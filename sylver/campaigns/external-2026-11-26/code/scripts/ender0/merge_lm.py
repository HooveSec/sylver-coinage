#!/usr/bin/env python3
# Memory-safe Q' builder (Sylver worker 2026-10-27), replaces merge.py + dedup_qprime.py for big roots (those load everything into Python sets /
# full numpy copies: ~5+ GB here). Q' = Q (fin3 journal, unchanged, same order) + every key of every ender0 P-journal j_*.bin that is not in Q,
# sorted, one 16-byte record each (15-byte Kunz key + 0 pad). Every ender child contains 16, so its journal is already keyed mod 16 (M=16
# per header, merge.py's rekey is the identity for M=16; the few small M<16 journals are re-keyed with merge.py's exact rekey()). numpy 'S15'
# holds the 15 key bytes (fixed width, compared bytewise = Python bytes order), so output == merge.py+dedup_qprime.py byte for byte (tested on 68+257).
# Peak ~ 3 x 15 B x (#journal records) + 64 MB chunks. usage: merge_lm.py Q.bin OUT.bin [journal dir]
import sys, os, glob, re, numpy as np
Q, OUT = sys.argv[1], sys.argv[2]; D = sys.argv[3] if len(sys.argv) > 3 else '.'
def rekey(k, m):   # verbatim from merge.py
    w=[0]+[r+m*k[r-1] for r in range(1,m)]; inS=lambda n:n>=w[n%m]
    out=bytearray(16)
    for r in range(1,16):
        n=r
        while not inS(n): n+=16
        out[r-1]=(n-r)//16
    return bytes(out)
js = sorted(glob.glob(os.path.join(D, 'j_*.bin'))); n = 0; M = {}
for j in js:
    h = open(j, 'rb').read(64).rstrip(b'\0').decode(); M[j] = int(re.search(r'M=(\d+)', h).group(1))
    n += (os.path.getsize(j) - 64) // 16
ex = np.empty(n, dtype='S15'); o = 0
for j in js:
    if M[j] != 16:
        d = open(j, 'rb').read(); a = np.frombuffer(b''.join(rekey(d[i:i+15], M[j]) for i in range(64, len(d)-15, 16)), dtype=np.uint8).reshape(-1, 16)
    else:
        a = np.fromfile(j, dtype=np.uint8, offset=64); a = a[:len(a) // 16 * 16].reshape(-1, 16)
    ex[o:o + len(a)] = np.ascontiguousarray(a[:, :15]).view('S15').ravel(); o += len(a); del a
assert o == n
ex = np.unique(ex); inq = np.zeros(len(ex), dtype=bool); nq = 0; CH = 4_000_000
qh = open(Q, 'rb').read(64)
with open(Q, 'rb') as f:
    f.seek(64)
    while True:
        b = f.read(16 * CH)
        if not b: break
        c = np.ascontiguousarray(np.frombuffer(b, dtype=np.uint8).reshape(-1, 16)[:, :15]).view('S15').ravel(); nq += len(c)
        p = np.searchsorted(ex, c); ok = p < len(ex); p2 = p[ok]; hit = ex[p2] == c[ok]; inq[p2[hit]] = True
keep = ex[~inq]; del ex, inq
tmp = OUT + '.tmp'
with open(tmp, 'wb') as g, open(Q, 'rb') as f:
    while True:
        b = f.read(1 << 26)
        if not b: break
        g.write(b)
    r = np.zeros((len(keep), 16), dtype=np.uint8); r[:, :15] = keep.view(np.uint8).reshape(-1, 15); g.write(r.tobytes())
os.replace(tmp, OUT)
print(OUT, 'journals', len(js), 'journal records', n, 'Q', nq, 'new extra keys', len(keep), "|Q'|", nq + len(keep))
