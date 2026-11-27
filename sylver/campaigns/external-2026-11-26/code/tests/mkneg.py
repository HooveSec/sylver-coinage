import sys
def S(gens,L=2048):
    s=[False]*L; s[0]=True
    for n in range(1,L): s[n]=any(n>=g and s[n-g] for g in gens)
    return s
def key(gens,M):
    s=S(gens); k=bytearray(15)
    for r in range(1,M):
        w=r
        while not s[w]: w+=M
        k[r-1]=(w-r)//M
    if M<16: k[14]=0xF0+M
    return bytes(k)
src='j_11_16_58.bin'; b=open(src,'rb').read(); H=b[:64]
recs=[b[64+16*i:80+16*i] for i in range((len(b)-64)//16)]
R=[11,16,58]; M=11; rk=key(R,M)
def W(name,rs): open(name,'wb').write(H+b''.join(rs))
pad=lambda k:k+b'\0'
W('neg_addN.bin',recs+[pad(key(R+[12],M))])                     # false P: child of the P root
W('neg_noroot.bin',[r for r in recs if r[:15]!=rk])
W('neg_drop.bin',[r for i,r in enumerate(recs) if i%10!=3 or r[:15]==rk])
bad=bytearray(rk); bad[0]=0; bad[1]=200                          # Kunz closure violated (w1=1 in S but w2 huge)
W('neg_badkunz.bin',recs+[pad(bytes(bad))])
W('neg_noroot_contain.bin',recs+[pad(key([11,17,23,29,35,41,47,53,59,65],M))])  # valid semigroup, but 16 not in it -> invalid
W('neg_termchild.bin',recs+[pad(key([3,4,5],M))])                # gaps {1,2}: move 2 reaches the terminal -> N
W('pos_terminal.bin',recs+[pad(key([2,3],M))])                   # gaps {1}: terminal P, allowed
W('pos_dup.bin',recs+recs[:50])                                  # duplicates: still OK
fl=bytearray(recs[len(recs)//2]); fl[3]^=1; W('neg_flip.bin',recs+[bytes(fl)])  # a perturbed key (valid or invalid; must not pass if N)
