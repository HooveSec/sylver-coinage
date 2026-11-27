// fin3: exact solver for FINITE Sylver coinage positions (gcd 1), built for 1e9-state cases under ~3 GB RAM.
// State = Apery set mod M (M = smallest generator, M<=16); key = 15-byte Kunz coordinates (full key, exact compare).
// Memory design (PRUNING.md R5 "store only P"; Bras-Amoros-Moskowitz-Ponomarenko 2025 §4; Cilibrasi --p-table):
//  * P-table: every P-position found is kept forever in a RAM open-addressing table of 16-byte records (PBITS)
//    and appended to a journal file (PJOURNAL) = checkpoint; on restart the journal is reloaded (torn tail dropped).
//  * N-positions go only to a lossy direct-mapped cache (NBITS); an N-position is re-proved cheaply by ETC:
//    scan its children for one in the P-table (a P child = winning move).
//  * Values are exact: a P is recorded only after ALL children were evaluated N; N only after a P child (or rule) found.
// Rules (PRUNING.md): R0 t<=1 => P.  R1 (ENDER=1, Hutchings / Sicherman Integers 2 (2002) G02 §1,§3 Weak Antisymmetry):
//    t>1 and no m<n, m+n=t, both gaps => ender => N.  Nothing else (no lemmas).
use std::env; use std::time::Instant; use std::fs::OpenOptions; use std::io::{Read,Write,BufWriter};
const MX:usize=16; type W=[u32;MX]; type K=[u8;15];
static mut MM:usize=16; fn md()->usize{ unsafe{MM} }
fn gcd(a:usize,b:usize)->usize{ if b==0 {a} else {gcd(b,a%b)} }
fn wadd(w:&W,a:usize)->W{ let m=md(); let mut v=*w; let ar=a%m; if ar==0 {return v;} if (a as u32)<v[ar]{v[ar]=a as u32;}
  let d=gcd(ar,m); let l=m/d; let au=a as u32;
  for s in 0..d { let mut r=s; for _ in 0..2*l { let n=r+ar; let n=if n>=m {n-m} else {n}; let c=v[r]+au; if c<v[n]{v[n]=c;} r=n; } } v }
fn key(w:&W)->K{ let m=md(); let mut k=[0u8;15]; for r in 1..m { let q=(w[r] as usize-r)/m; assert!(q<240,"Kunz coordinate overflow"); k[r-1]=q as u8; } if m<16 {k[14]=0xF0+m as u8;} k }
fn hk(k:&K)->u64{ let mut a=[0u8;16]; a[..15].copy_from_slice(k); let x=u64::from_le_bytes(a[..8].try_into().unwrap()); let y=u64::from_le_bytes(a[8..].try_into().unwrap());
  let mut h=x.wrapping_mul(0x9E3779B97F4A7C15)^y.rotate_left(29).wrapping_mul(0xC2B2AE3D27D4EB4F); h^=h>>31; h=h.wrapping_mul(0xFF51AFD7ED558CCD); h^=h>>33; h=h.wrapping_mul(0xC4CEB9FE1A85EC53); h^h>>29 }
fn chk(h:u64)->u8{ 0x80 | (((h>>40) as u8)&0x7e) }   // same record format as even2 store, value bit 0 = P
struct S{ pt:Vec<[u8;16]>, pmask:u64, pn:u64, pcap:u64, pfull:bool, nc:Vec<[u8;16]>, nmask:u64, j:Option<BufWriter<std::fs::File>>,
  nodes:u64, hit_p:u64, hit_n:u64, etc_win:u64, ender:u64, killer:Vec<u32>, use_ender:bool, t0:Instant, last:f64, last_n:u64, best:f64, rate_sec:f64, last_flush:f64 }
impl S{
  fn pget(&self,k:&K,h:u64)->bool{ let mut i=h&self.pmask; loop { let e=&self.pt[i as usize]; if e[15]==0 {return false;} if &e[..15]==&k[..] {return true;} i=(i+1)&self.pmask; } }
  fn pins(&mut self,k:&K,h:u64,journal:bool){ if self.pfull {return;} if self.pn>=self.pcap { self.pfull=true; eprintln!("WARNING P-table at 85%: further P kept nowhere (still correct, slower)"); return; }
    let mut i=h&self.pmask; loop { let e=&self.pt[i as usize]; if e[15]==0 {break;} if &e[..15]==&k[..] {return;} i=(i+1)&self.pmask; }
    let mut rec=[0u8;16]; rec[..15].copy_from_slice(k); rec[15]=chk(h); self.pt[i as usize]=rec; self.pn+=1;
    if journal { if let Some(j)=&mut self.j { j.write_all(&rec).unwrap(); } } }
  fn nget(&self,k:&K,h:u64)->bool{ let e=&self.nc[(h&self.nmask) as usize]; e[15]!=0 && &e[..15]==&k[..] }
  fn nput(&mut self,k:&K,h:u64){ let e=&mut self.nc[(h&self.nmask) as usize]; e[..15].copy_from_slice(k); e[15]=1; } }
fn rss()->String{ std::fs::read_to_string("/proc/self/status").unwrap_or_default().lines().filter(|l|l.starts_with("VmRSS")||l.starts_with("VmHWM")).map(|l|l.split_whitespace().collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join(" ") }
fn tick(s:&mut S){ let t=s.t0.elapsed().as_secs_f64();
  if t-s.last_flush>60.0 { if let Some(j)=&mut s.j { j.flush().unwrap(); } s.last_flush=t; }
  if t-s.last<s.rate_sec {return;} let rate=(s.nodes-s.last_n) as f64/(t-s.last); if rate>s.best {s.best=rate;}
  eprintln!("[rate] t={:.0}s nodes={} {:.0} nodes/s P={} hitP={} hitN={} etc_win={} ender={} {}",t,s.nodes,rate,s.pn,s.hit_p,s.hit_n,s.etc_win,s.ender,rss());
  if rate<0.3*s.best { eprintln!("WATCHDOG WARNING: rate {:.0} < 30% of best {:.0}",rate,s.best); } s.last=t; s.last_n=s.nodes; }
fn is_ender(w:&W,t:usize)->bool{ let m=md(); let gap=|x:usize| x<w[x%m] as usize; let mut a=1; while 2*a<t { if gap(a)&&gap(t-a) {return false;} a+=1; } true }
// returns true iff the player to move wins (N)
fn win(s:&mut S,w:&W)->bool{ let m=md(); let f=*w[..m].iter().max().unwrap() as usize-m; if f<=1 {return false;}
  let k=key(w); let h=hk(&k);
  if s.pget(&k,h){ s.hit_p+=1; return false; } if s.nget(&k,h){ s.hit_n+=1; return true; }
  if s.use_ender && is_ender(w,f) { s.ender+=1; s.nput(&k,h); return true; }
  s.nodes+=1; if s.nodes&0xFFFF==0 { tick(s); }
  // ETC: a child already known P (or terminal P) proves N without recursion
  let mut x=f; while x>=2 { if x<w[x%m] as usize { let c=wadd(w,x); let cf=*c[..m].iter().max().unwrap() as usize-m;
      if cf<=1 { s.etc_win+=1; s.nput(&k,h); return true; } let ck=key(&c); if s.pget(&ck,hk(&ck)) { s.etc_win+=1; s.killer[f]=x as u32; s.nput(&k,h); return true; } } x-=1; }
  let kx=s.killer[f] as usize;
  if kx>=2 && kx<=f && kx<w[kx%m] as usize { if !win(s,&wadd(w,kx)) { s.nput(&k,h); return true; } }
  let mut x=f; while x>=2 { if x!=kx && x<w[x%m] as usize { if !win(s,&wadd(w,x)) { s.killer[f]=x as u32; s.nput(&k,h); return true; } } x-=1; }
  s.pins(&k,h,true); false }
fn main(){ let v:Vec<usize>=env::args().skip(1).map(|x|x.parse().unwrap()).collect(); assert!(v.len()>=2);
  let g=v.iter().fold(0,|a,&b|gcd(a,b)); assert!(g==1,"finite positions only (gcd 1)"); let m=*v.iter().min().unwrap(); assert!(m>=2&&m<=16); unsafe{MM=m;}
  let ev=|n:&str,d:u64|->u64{ env::var(n).ok().and_then(|x|x.parse().ok()).unwrap_or(d) };
  let pbits=ev("PBITS",26); let nbits=ev("NBITS",24);
  let mut w=[0u32;MX]; for r in 1..m { w[r]=u32::MAX; } for &a in &v { w=if a%m==0 {w} else { let mut t=w; let ar=a%m; if (a as u32)<t[ar]{t[ar]=a as u32;} t }; }
  // proper closure: start from {m}, add generators one by one
  let mut w0=[0u32;MX]; for r in 1..m { w0[r]=u32::MAX/2; } let mut wc=w0; for &a in &v { if a!=m { wc=wadd(&wc,a);} } assert!(wc[..m].iter().all(|&x|x<u32::MAX/2),"not finite"); let _=w; let w=wc;
  let mut s=S{pt:vec![[0u8;16];1usize<<pbits],pmask:(1u64<<pbits)-1,pn:0,pcap:((1u64<<pbits) as f64*0.85) as u64,pfull:false,nc:vec![[0u8;16];1usize<<nbits],nmask:(1u64<<nbits)-1,j:None,
    nodes:0,hit_p:0,hit_n:0,etc_win:0,ender:0,killer:vec![0u32;4096],use_ender:ev("ENDER",1)==1,t0:Instant::now(),last:0.0,last_n:0,best:0.0,rate_sec:ev("RATE_SEC",300) as f64,last_flush:0.0};
  if let Ok(p)=env::var("PJOURNAL") { let tag=format!("{:?}",v);
    // journal starts with a 64-byte header naming the position: refuse to mix journals of different roots (sound anyway, but keeps runs tidy)
    if let Ok(mut f)=std::fs::File::open(&p) { let mut hdr=[0u8;64];
      // 2026-10-23: stream the journal in 16 MB chunks (was read_to_end: whole file + Vec growth pushed VSZ past the 2.9 GB ulimit)
      let flen=f.metadata().unwrap().len() as usize; let mut hn=0usize; while hn<64.min(flen) { let r=f.read(&mut hdr[hn..64.min(flen)]).unwrap(); if r==0 {break;} hn+=r; }
      if flen>=64 { let ht=String::from_utf8_lossy(&hdr).trim_end_matches('\0').to_string(); eprintln!("journal header: {}",ht); }
      let blen=flen.saturating_sub(64); let whole=blen/16*16;
      let mut ok=0u64; let mut bad=0u64; let mut done=0usize; let mut chunk=vec![0u8;1<<24];
      while done<whole { let want=(whole-done).min(1<<24); f.read_exact(&mut chunk[..want]).unwrap();
        for rec in chunk[..want].chunks_exact(16){ let mut k=[0u8;15]; k.copy_from_slice(&rec[..15]); let h=hk(&k); if rec[15]==chk(h) { s.pins(&k,h,false); ok+=1; } else { bad+=1; } }
        done+=want; }
      let body_len=blen;
      eprintln!("journal {}: reloaded {} P records ({} bad, {} torn tail bytes)",p,ok,bad,body_len-whole);
      let fl=OpenOptions::new().write(true).open(&p).unwrap(); fl.set_len((64+whole) as u64).unwrap(); }
    let mut f=OpenOptions::new().create(true).append(true).open(&p).unwrap();
    if f.metadata().unwrap().len()==0 { let mut hdr=[0u8;64]; let b=format!("fin3 {} M={}",tag,m); let n=b.len().min(63); hdr[..n].copy_from_slice(&b.as_bytes()[..n]); f.write_all(&hdr).unwrap(); }
    s.j=Some(BufWriter::with_capacity(1<<20,f)); }
  eprintln!("fin3 {:?} M={} F={} PBITS={} NBITS={} ender={} start",v,m,*w[..m].iter().max().unwrap() as usize-m,pbits,nbits,s.use_ender);
  let handle=std::thread::Builder::new().stack_size(512<<20).spawn(move||{ let r=win(&mut s,&w); if let Some(j)=&mut s.j { j.flush().unwrap(); } (r,s) }).unwrap();
  let (r,s)=handle.join().unwrap();
  eprintln!("[final] nodes={} P={} hitP={} hitN={} etc_win={} ender={} {}",s.nodes,s.pn,s.hit_p,s.hit_n,s.etc_win,s.ender,rss());
  println!("{:?} F={} {} nodes={} P={} {:.2}s fin3 ender={}",v,*w[..m].iter().max().unwrap() as usize-m,if r {"N"} else {"P"},s.nodes,s.pn,s.t0.elapsed().as_secs_f64(),s.use_ender); }
