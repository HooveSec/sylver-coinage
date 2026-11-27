#!/usr/bin/env python3
# Build Q' = Q u (every ender0 journal re-keyed mod 16) as a fin3-format journal for root {16,38,68,72,257}, for the final ENDER=0 re-check.
# Each extra key must still contain the root (it does: it lies in an ender child's game); the checkers re-verify everything anyway.
import glob,sys,re,shutil
Q='/workspace/sylver/fin3/j_38_68_72_257.bin'; OUT=sys.argv[1] if len(sys.argv)>1 else 'qprime_16_38_68_72_257.bin'
def rekey(k,m):
    w=[0]+[r+m*k[r-1] for r in range(1,m)]; inS=lambda n:n>=w[n%m]
    out=bytearray(16)
    for r in range(1,16):
        n=r
        while not inS(n): n+=16
        out[r-1]=(n-r)//16
    return bytes(out)
extra=set()
for j in sorted(glob.glob('j_*.bin')):
    d=open(j,'rb').read(); h=d[:64].rstrip(b'\0').decode(); m=int(re.search(r'M=(\d+)',h).group(1))
    for i in range(64,len(d)-15,16): extra.add(rekey(d[i:i+15],m))
shutil.copyfile(Q,OUT)
with open(OUT,'ab') as f:
    for k in sorted(extra): f.write(k)
print(OUT,'extra keys',len(extra))
