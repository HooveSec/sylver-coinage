// checkq2: SECOND, independent P-set certificate checker (Sylver worker, 2026-10-19). Rust; shares no code with checkq.c, fin3, even3, pattern2.
// Representation differs from checkq.c on purpose:
//  * positions are Apery vectors w[0..M) (w_r = least element of S that is = r mod M); a move is the Apery-vector update
//    w'_r = min_{0<=j<M} ( w_{(r - j a) mod M} + j a )   (S + aN; j >= M never helps since M*a = 0 mod M only adds), NOT a gap bitmap;
//  * gaps are enumerated from w (n is a gap iff n < w_{n mod M}); F = max_r w_r - M;
//  * Q is a STATIC table: keys as u128 (15 bytes big-endian), sorted by (mix(key), key) with a CSR bucket index on the top bits of mix,
//    NOT an open-addressing hash; the journal header is parsed into an integer list and compared with argv as integers.
// Rules (Sylver coinage): position = numerical semigroup of the numbers named; legal move = any gap; naming 1 loses at once, so a position whose
// only gap is 1 is P (terminal) and 1 is never a winning move. CLAIM: every X in Q is P. For every X in Q with F(X) > 1 and every gap a >= 2:
// Y = X + a must have F(Y) > 1 (else X has a P child: FAIL) and some gap y >= 2 with F(Y+y) <= 1 or key(Y+y) in Q
// (or, ENDER=1, Y an ender: F(Y) > 1 and no gaps p < q with p + q = F(Y); Hutchings, Sicherman Integers 2 (2002) G02). Root in Q, gcd(root) = 1.
// Induction on the number of gaps => every X in Q is P, so the root is P.
// Env: ENDER=1, ENDERLOG=file (15-byte keys + 0 pad, one per ender use), THREADS (default 1), CKPT=file + CKSTEP (default 1,000,000 keys),
//      SAMPLE=n SEED=s (check n random X only: timing / spot check, never a certificate). Exit: 0 OK, 1 FAIL, 2 ERROR, 3 PARTIAL.
use std::env; use std::fs; use std::io::{Read,Write}; use std::sync::atomic::{AtomicU64,AtomicBool,Ordering::Relaxed}; use std::time::Instant;
type Ap=[u32;16];
fn err(m:&str)->!{ println!("ERROR {}",m); std::process::exit(2) }
fn gcd(a:u32,b:u32)->u32{ if b==0 {a} else {gcd(b,a%b)} }
fn mix(x:u128)->u64{ let a=(x as u64)^((x>>64) as u64).rotate_left(23); let mut z=a.wrapping_add(0x9E3779B97F4A7C15); z=(z^(z>>30)).wrapping_mul(0xBF58476D1CE4E5B9); z=(z^(z>>27)).wrapping_mul(0x94D049BB133111EB); z^(z>>31) }
struct Q{ keys:Vec<u128>, start:Vec<u32>, shift:u32 }
impl Q{ fn has(&self,k:u128)->bool{ let b=(mix(k)>>self.shift) as usize; let (s,e)=(self.start[b] as usize,self.start[b+1] as usize); self.keys[s..e].iter().any(|&x|x==k) } }
struct G{ m:usize, root:Vec<u32> }
impl G{
  fn frob(&self,w:&Ap)->i64{ (*w[..self.m].iter().max().unwrap() as i64)-(self.m as i64) }
  fn gap(&self,w:&Ap,n:u32)->bool{ n < w[n as usize % self.m] }
  fn add(&self,w:&Ap,a:u32)->Ap{ let m=self.m; let mut v=*w; let ar=a as usize%m;
    for r in 0..m { let mut best=w[r]; let mut src=r; for j in 1..m as u32 { let ja=j*a; if ja>=best {break;} src=if src>=ar {src-ar} else {src+m-ar}; let c=w[src]+ja; if c<best {best=c;} } v[r]=best; } v }
  fn enc(&self,w:&Ap)->u128{ let m=self.m; let mut b=[0u8;16]; for r in 1..m { let q=(w[r]-r as u32)/m as u32; if q>=240 {err("coordinate overflow");} b[r]=q as u8; } if m<16 {b[15]=0xF0+m as u8;} u128::from_be_bytes(b) }
  // decode: b[1..16] = 15 key bytes; returns None unless canonical, Kunz-closed, and contains the root
  fn dec(&self,k:u128)->Option<Ap>{ let m=self.m; let b=k.to_be_bytes(); if m<16 { if b[15]!=0xF0+m as u8 {return None;} if b[m..15].iter().any(|&x|x!=0) {return None;} }
    let mut w=[0u32;16]; for r in 1..m { w[r]=r as u32+(m as u32)*(b[r] as u32); }
    for i in 1..m { for j in 1..m { if w[i]+w[j] < w[(i+j)%m] {return None;} } }
    for &g in &self.root { if g < w[g as usize % m] {return None;} } Some(w) }
  fn ender(&self,w:&Ap)->bool{ let t=self.frob(w); if t<=1 {return false;} let t=t as u32; for p in 1..t { let q=t-p; if p>=q {break;} if self.gap(w,p)&&self.gap(w,q) {return false;} } true }
}
fn main(){ let a:Vec<String>=env::args().collect(); if a.len()<3 { eprintln!("usage: checkq2 journal g1 g2 ..."); std::process::exit(2); }
  let ev=|k:&str,d:u64|->u64{ env::var(k).ok().and_then(|x|x.parse().ok()).unwrap_or(d) };
  let use_ender=ev("ENDER",0)==1; let threads=ev("THREADS",1).max(1) as usize; let ckstep=ev("CKSTEP",1_000_000).max(1); let sample=ev("SAMPLE",0); let seed=ev("SEED",1);
  let mut root:Vec<u32>=Vec::new(); for s in &a[2..] { match s.parse::<u32>() { Ok(g) if g>=2 && g<4096 => root.push(g), _ => err("bad generator") } }
  if root.iter().fold(0,|x,&y|gcd(x,y))!=1 { err("root gcd != 1: unbounded game, not a finite certificate"); }
  let m=*root.iter().min().unwrap() as usize; if m>16 { err("smallest generator > 16"); }
  let g=G{m,root:root.clone()};
  // streamed load (peak RAM ~ |Q| x 16 B + index): header first, then 16-byte records in 1 MiB chunks
  let mut f=match fs::File::open(&a[1]) { Ok(f)=>f, Err(_)=>err("cannot open journal") }; let flen=f.metadata().map(|m|m.len()).unwrap_or(0) as usize;
  let mut raw=vec![0u8;64]; if flen<64 || f.read_exact(&mut raw).is_err() { err("short header"); }
  // header "fin3 [g1, g2, ...] M=m": parse to integers
  { let h=String::from_utf8_lossy(&raw[..64]).trim_end_matches('\0').to_string();
    let ok=(||{ let r=h.strip_prefix("fin3 [")?; let (lst,rest)=r.split_once(']')?; let hm:usize=rest.trim().strip_prefix("M=")?.trim().parse().ok()?;
      let v:Vec<u32>=lst.split(',').map(|x|x.trim().parse::<u32>()).collect::<Result<_,_>>().ok()?; Some(v==root && hm==m) })();
    if ok!=Some(true) { err("journal header does not name this root/M"); } }
  // root Apery vector from scratch (shortest path on residues mod m)
  let mut rw=[u32::MAX;16]; rw[0]=0; loop { let mut ch=false; for r in 0..m { if rw[r]==u32::MAX {continue;} for &x in &root { let n=(r+x as usize)%m; let c=rw[r]+x; if c<rw[n] {rw[n]=c; ch=true;} } } if !ch {break;} }
  let rf=g.frob(&rw); if rf<0 || rf>2000 { err("root Frobenius out of range"); }
  let nrec=(flen-64)/16; let mut keys:Vec<u128>=Vec::with_capacity(nrec); let mut buf=vec![0u8;1<<20];
  while keys.len()<nrec { let want=((nrec-keys.len())*16).min(buf.len()); if f.read_exact(&mut buf[..want]).is_err() { err("read"); }
    for rec in buf[..want].chunks_exact(16) { let mut b=[0u8;16]; b[1..16].copy_from_slice(&rec[..15]); keys.push(u128::from_be_bytes(b)); } }
  drop(buf); drop(f);
  let mut bits=1u32; while (1u64<<bits) < (nrec as u64/2).max(2) { bits+=1; } let shift=64-bits; let nb=1usize<<bits;
  keys.sort_unstable_by_key(|&k|(mix(k)>>shift,k)); let n0=keys.len(); keys.dedup(); let dup=n0-keys.len(); keys.shrink_to_fit();
  let mut start=vec![0u32;nb+1]; for &k in &keys { start[(mix(k)>>shift) as usize+1]+=1; } for i in 0..nb { start[i+1]+=start[i]; }
  let q=Q{keys,start,shift}; let nq=q.keys.len();
  if !q.has(g.enc(&rw)) { println!("FAIL root not in Q"); std::process::exit(1); }
  // which indices to check
  let idx:Vec<u32>= if sample>0 { let mut s=seed.wrapping_mul(0x9E3779B97F4A7C15)|1; (0..sample.min(nq as u64)).map(|_|{ s^=s<<13; s^=s>>7; s^=s<<17; (s%nq as u64) as u32 }).collect() } else { Vec::new() };
  let total= if sample>0 {idx.len()} else {nq};
  let (mut pos,mut ch0,mut pr0,mut en0)=(0usize,0u64,0u64,0u64); let ck=env::var("CKPT").ok();
  if let (Some(c),0)=(&ck,sample) { if let Ok(s)=fs::read_to_string(c) { let v:Vec<u64>=s.split_whitespace().filter_map(|x|x.parse().ok()).collect();
      if v.len()==6 { if v[0]!=nrec as u64 || v[1]!=nq as u64 { err("checkpoint is for a different journal"); } pos=v[2] as usize; ch0=v[3]; pr0=v[4]; en0=v[5]; eprintln!("[checkq2] resume at {}/{}",pos,nq); } } }
  eprintln!("[checkq2] root={:?} M={} F={} records={} |Q|={} dup={} buckets=2^{} ender={} threads={} sample={} from={}",root,m,rf,nrec,nq,dup,bits,use_ender,threads,sample,pos);
  let (children,probes,enders,termw)=(AtomicU64::new(ch0),AtomicU64::new(pr0),AtomicU64::new(en0),AtomicU64::new(0)); let fail=AtomicBool::new(false);
  let elog=std::sync::Mutex::new(env::var("ENDERLOG").ok().map(|p|fs::OpenOptions::new().create(true).append(true).open(p).unwrap_or_else(|_|err("enderlog"))));
  let t0=Instant::now();
  let check=|xi:usize,kill:&mut Vec<u32>|->Result<(),String>{ let x=q.keys[xi]; let w=g.dec(x).ok_or(format!("invalid key (sorted index {})",xi))?; let t=g.frob(&w); if t<=1 {return Ok(());}
    let (mut lc,mut lp)=(0u64,0u64);
    for a in 2..=t as u32 { if !g.gap(&w,a) {continue;} let y=g.add(&w,a); lc+=1; let ty=g.frob(&y);
      if ty<=1 { return Err(format!("key {} move {} reaches the terminal P position",xi,a)); }
      let memo=kill[a as usize]; let mut ok=false; let tryr=|r:u32,lp:&mut u64|->bool{ *lp+=1; let z=g.add(&y,r); g.frob(&z)<=1 || q.has(g.enc(&z)) };
      if memo>=2 && (memo as i64)<=ty && g.gap(&y,memo) && tryr(memo,&mut lp) { ok=true; }
      if !ok { for r in (2..=ty as u32).rev() { if r==memo || !g.gap(&y,r) {continue;} if tryr(r,&mut lp) { ok=true; kill[a as usize]=r; break; } } }
      if !ok && use_ender && g.ender(&y) { ok=true; enders.fetch_add(1,Relaxed); if let Some(f)=elog.lock().unwrap().as_mut() { let b=g.enc(&y).to_be_bytes(); let mut r=[0u8;16]; r[..15].copy_from_slice(&b[1..16]); f.write_all(&r).unwrap(); } }
      if !ok { return Err(format!("key {} move {}: child has no reply into Q{}",xi,a,if use_ender {""} else {" (ENDER=0)"})); } }
    children.fetch_add(lc,Relaxed); probes.fetch_add(lp,Relaxed); Ok(()) };
  while pos<total { let end=(pos+ckstep as usize).min(total); let next=AtomicU64::new(pos as u64); let msg=std::sync::Mutex::new(None::<String>);
    std::thread::scope(|s|{ for _ in 0..threads { s.spawn(||{ let mut kill=vec![0u32;4096]; loop { if fail.load(Relaxed) {break;} let i=next.fetch_add(256,Relaxed) as usize; if i>=end {break;}
          for j in i..(i+256).min(end) { let xi= if sample>0 {idx[j] as usize} else {j}; if let Err(e)=check(xi,&mut kill) { fail.store(true,Relaxed); *msg.lock().unwrap()=Some(e); break; } } } }); } });
    if let Some(e)=msg.into_inner().unwrap() { println!("FAIL {}",e); std::process::exit(1); }
    pos=end; if let Some(f)=elog.lock().unwrap().as_mut() { f.flush().unwrap(); }
    if let (Some(c),0)=(&ck,sample) { let tmp=format!("{}.tmp",c); fs::write(&tmp,format!("{} {} {} {} {} {}\n",nrec,nq,pos,children.load(Relaxed),probes.load(Relaxed),enders.load(Relaxed))).unwrap(); fs::rename(&tmp,c).unwrap(); }
    eprintln!("[checkq2] {}/{} children={} probes={} enders={} {:.0}s",pos,total,children.load(Relaxed),probes.load(Relaxed),enders.load(Relaxed),t0.elapsed().as_secs_f64());
    let stop=ev("STOPAFTER",0); if stop>0 && pos as u64>=stop && pos<total { println!("PARTIAL stopped at {} (checkpointed; not a certificate)",pos); std::process::exit(3); } }
  let (c,p,e)=(children.load(Relaxed),probes.load(Relaxed),enders.load(Relaxed));
  if sample>0 { println!("PARTIAL sample={} seed={} ok children={} probes={} enders={} (not a certificate)",total,seed,c,p,e); std::process::exit(3); }
  println!("OK root={:?} F={} M={} |Q|={} dup={} children={} probes={} enders={} => root is P{}",root,rf,m,nq,dup,c,p,e,if e>0 {" (uses the ender theorem)"} else {" (no ender theorem used)"});
  let _=termw; }
