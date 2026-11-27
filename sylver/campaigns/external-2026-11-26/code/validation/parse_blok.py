import re,sys
from math import gcd
from functools import reduce
docs=[('../refs/6-16Tables.txt','Blok 6-16 Tables (2026)','text'),
      ('../refs/Sylver_Coinage_positions_with_g=2.txt','Blok g=2 (2021)',{8,10}),
      ('../refs/Sylver_Coinage_even_positions_in_14.txt','Blok 14-even (2022)',{14})]
TOK=re.compile(r'\{[^}]*\}|\[[^\]]*\]?|𝓟|X|\?|[\d][\d,\.]*')
def nums(s): return [int(x) for x in re.findall(r'\d+',s)]
def isText(l): 
    t=l.strip()
    if re.match(r'\d+(\.\d+)?\s+\{',t): return True
    return bool(re.search(r'[A-Za-z]',t.replace('X',''))) or re.fullmatch(r'\d+',t) is not None
out={}; issues=[]
def add(pos,val,mv,src):
    pos=tuple(sorted(set(pos)))
    k=pos
    if k in out and out[k][0]!=val: issues.append(f"CONFLICT {k} {out[k]} vs {val} {src}")
    out.setdefault(k,(val,mv,src))
def entry(e):
    e=e.strip()
    if e=='𝓟' or e=='[]': return ('P','')
    if e in('X','?') or '?' in e: return None
    n=nums(e.replace('.',','))
    if not n: return None
    return ('N',','.join(map(str,n)))
for fn,src,base in docs:
    lines=open(fn,encoding='utf8').read().split('\n')
    hdr=None; prevtext=True; sec=''; seenh=set(); skipping=False
    for i,l in enumerate(lines):
        if not l.strip(): continue
        if re.match(r'\s*\d+(\.\d+)?\s+\S',l) and re.search(r'[A-Za-z{]',l) and not l.strip().startswith('{'): sec=l.strip()[:40]
        # inline statements
        txt=isText(l) or re.match(r'\s*\{[\d,\s]+\}:',l)
        for m in (re.finditer(r'\{([\d,\s]+)\}:?\s*\[(\d[\d,\.]*)',l) if txt else []): add(nums(m.group(1)),'N',','.join(map(str,nums(m.group(2)))),f"{src} inline L{i+1}")
        for m in re.finditer(r'\{([\d,\s]+)\},?\s*(?:which|is) (?:is )?a 𝓟',l): add(nums(m.group(1)),'P','',f"{src} inline L{i+1}")
        for m in re.finditer(r'\{([\d,\s]+)\} (?:is answered by|has a winning move of|has) (\d+)',l): add(nums(m.group(1)),'N',m.group(2),f"{src} text L{i+1}")
        if isText(l): prevtext=True; hdr=None; continue
        toks=[t for t in re.split(r'\s{2,}',l.strip()) if t]
        if len(toks)<2: continue
        first=toks[0]
        fs=set(nums(first)) if first!='{}' else set()
        ishdr = first.startswith('{') and all(not t.startswith('[') and t not in('𝓟','X') for t in toks[1:]) and (prevtext or (base!='text' and fs&base and min(fs)==min(base&fs)))
        if base=='text': ishdr = first.startswith('{') and prevtext and all(t.startswith('{') for t in toks)
        prevtext=False
        if ishdr:
            b=set(nums(first)); cols=[]
            for t in toks[1:]:
                c=set(nums(t)) if t not in('{}',) else set()
                if c==b: c=set()  # typo: base repeated as last column header
                cols.append(c)
            key=(fn,tuple(sorted(b)))
            if key in seenh: hdr=None; issues.append(f"duplicate table header {sorted(b)} at {fn}:{i+1}; table SKIPPED (likely typo, maybe {{14,32,34}})"); skipping=True; continue
            seenh.add(key); skipping=False
            rawcols=[set(nums(t)) if t!='{}' else set() for t in l.strip().split()[1:]]
            hdr=(b,cols,rawcols); continue
        if hdr is None and skipping: continue
        if hdr is None: issues.append(f"orphan row {fn}:{i+1}: {l.strip()}"); continue
        b,cols,rawcols=hdr; r=set(nums(first)) if first!='{}' else set()
        ents=toks[1:]
        if len(ents)!=len(cols) and len(l.split())-1==len(rawcols): cols=rawcols; ents=l.split()[1:]
        if len(ents)!=len(cols): issues.append(f"colcount {fn}:{i+1} hdr={sorted(b)} {len(ents)}!={len(cols)}: {l.strip()}"); continue
        for c,e in zip(cols,ents):
            v=entry(e)
            if v: add(b|r|c,v[0],v[1],f"{src} table {sorted(b)} row {sorted(r)} col {sorted(c)} L{i+1}")
with open('../known_blok.tsv','w') as f:
    f.write("position\toutcome\twinning_moves\tsource\n")
    for k,(v,mv,s) in sorted(out.items(),key=lambda x:(len(x[0]),x[0])): f.write(f"{','.join(map(str,k))}\t{v}\t{mv}\t{s}\n")
print(len(out),"positions;",sum(v=='P' for v,_,_ in out.values()),"P")
for s in issues: print(s)
