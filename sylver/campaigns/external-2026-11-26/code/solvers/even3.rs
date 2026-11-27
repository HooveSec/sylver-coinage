// even3: even2 with the same game logic, keys, hash and store FILE FORMAT (store16.bin compatible), plus:
//  * RAMIDX=<bits>: whole store index held in RAM (16-byte records, full key -> exact), so lookups never touch disk;
//    new records are buffered and flushed to the file sorted by slot (FLUSH records per batch) -> no random 4 KB
//    read-modify-write per 16-byte record. Without RAMIDX: even2 behaviour (1-byte fp array + pread) with
//    posix_fadvise(RANDOM) so a 16-byte probe no longer pulls 128 KB of readahead.
//  * BASE=<path> BASE_BITS=<b>: optional READ-ONLY lower layer (another job's store, e.g. store16.bin). Never written.
//    Same key space (Kunz coords mod M=16), and a finite position's value does not depend on its parent -> reuse is sound.
//  * wadd in O(M) (two passes per residue cycle) instead of O(M^2).  KILLER=1: try the move that last won at the same
//    Frobenius number first (ordering only; values unchanged).  ETC=1: before recursing, scan children for a RAM-known
//    losing child.
//  * pattern2 failures are counted and reported (pfail=) and FATAL if PFAIL_HARD=1 (exit 3).
//  * DEADLINE=<s>: graceful timeout (flush store, print TIMEOUT, exit 4); TIMEOUT is never cached.
//  * watchdog: every RATE_SEC s prints nodes/s and disk read MB/s; WARNs if the rate falls below 30% of the best window.
use std::collections::HashMap; use std::env; use std::time::Instant; use std::fs::{File,OpenOptions};
use std::os::unix::fs::FileExt; use std::os::unix::io::AsRawFd; use std::io::{Read,Write}; use std::hash::{BuildHasherDefault,Hasher};
extern "C" { fn posix_fadvise(fd:i32,off:i64,len:i64,adv:i32)->i32; }
fn fadv_random(f:&File){ unsafe{ posix_fadvise(f.as_raw_fd(),0,0,1); } }
type G=Vec<u64>; const MX:usize=16; type W=[u32;MX]; type K=[u8;15];
static mut MM:usize=16; fn md()->usize{ unsafe{MM} }
static mut PFAIL:u64=0;
fn get(g:&G,i:usize)->bool{ i<g.len()*64 && g[i>>6]>>(i&63)&1==1 }
fn frob(g:&G)->usize{ for w in (0..g.len()).rev(){ if g[w]!=0 {return w*64+63-g[w].leading_zeros() as usize;} } 0 }
fn from_in(inn:&[bool])->G{ let mut f=0; for i in 1..inn.len(){ if !inn[i]{f=i;} } let mut r=vec![0u64;f/64+1]; for i in 1..=f { if !inn[i]{ r[i>>6]|=1<<(i&63);} } r }
fn add(g:&G,a:usize)->G{ let f=frob(g); let mut inn=vec![true;f+1]; for i in 0..=f { if get(g,i){inn[i]=false;} }
  if a<=f {inn[a]=true;} for n in a..=f { if inn[n-a]{inn[n]=true;} } from_in(&inn) }
fn gens(v:&[usize])->G{ let m=*v.iter().min().unwrap(); let lim=v.iter().max().unwrap()*m+5; let mut inn=vec![false;lim]; inn[0]=true;
  for &a in v { for n in a..lim { if inn[n-a]{inn[n]=true;} } } from_in(&inn) }
// old O(M^2) version, kept for the self-test
fn wadd_old(w:&W,a:usize)->W{ let M=md(); let mut v=*w; let ar=a%M; if (a as u32)<v[ar]{v[ar]=a as u32;}
  for s in 0..M { let mut r=s; for _ in 0..M { let n=(r+ar)%M; let c=v[r]+a as u32; if c<v[n]{v[n]=c;} r=n; } } v }
fn gcd(a:usize,b:usize)->usize{ if b==0 {a} else {gcd(b,a%b)} }
fn wadd(w:&W,a:usize)->W{ let M=md(); let mut v=*w; let ar=a%M; if ar==0 {return v;} if (a as u32)<v[ar]{v[ar]=a as u32;}
  let d=gcd(ar,M); let l=M/d; let au=a as u32;
  for s in 0..d { let mut r=s; for _ in 0..2*l { let n=r+ar; let n=if n>=M {n-M} else {n}; let c=v[r]+au; if c<v[n]{v[n]=c;} r=n; } } v }
fn g2w(g:&G)->W{ let M=md(); assert!(!get(g,M),"modulus not in position"); let f=frob(g); let mut w=[0u32;MX]; for r in 0..M {w[r]=u32::MAX;} for x in 0..=f+M { let r=x%M; if w[r]==u32::MAX && !get(g,x){w[r]=x as u32;} }
  assert!(w[0]==0 && w[..M].iter().all(|&x|x!=u32::MAX),"bad apery"); w }
fn key(w:&W)->K{ let M=md(); let mut k=[0u8;15]; for r in 1..M { let q=(w[r] as usize-r)/M; assert!(q<240,"Kunz coordinate overflow"); k[r-1]=q as u8; } if M<16 {k[14]=0xF0+M as u8;} k }
fn hk(k:&K)->u64{ let mut a=[0u8;16]; a[..15].copy_from_slice(k); let x=u64::from_le_bytes(a[..8].try_into().unwrap()); let y=u64::from_le_bytes(a[8..].try_into().unwrap());
  let mut h=x.wrapping_mul(0x9E3779B97F4A7C15)^y.rotate_left(29).wrapping_mul(0xC2B2AE3D27D4EB4F); h^=h>>31; h=h.wrapping_mul(0xFF51AFD7ED558CCD); h^=h>>33; h=h.wrapping_mul(0xC4CEB9FE1A85EC53); h^h>>29 }
#[derive(Default)] struct Id(u64); impl Hasher for Id{ fn finish(&self)->u64{self.0} fn write(&mut self,b:&[u8]){ let mut k=[0u8;15]; if b.len()==15 {k.copy_from_slice(b); self.0=hk(&k);} else { for &x in b {self.0=(self.0.rotate_left(8)^x as u64).wrapping_mul(0x9E3779B97F4A7C15);} } } }
type C=HashMap<K,bool,BuildHasherDefault<Id>>;
fn chk(h:u64,v:bool)->u8{ 0x80 | (((h>>40) as u8)&0x7e) | v as u8 }
fn fpb(h:u64)->u8{ let b=(h>>56) as u8; if b==0 {1} else {b} }
// scan a store file; calls cb(slot,rec) for every valid record; returns (n,bad). Invalid records are zeroed only if rw.
fn scan(p:&str,slots:u64,rw:Option<&File>,mut cb:impl FnMut(u64,&[u8;16]))->(u64,u64){ let mut rd=File::open(p).unwrap(); let mut buf=vec![0u8;1<<24]; let mut slot=0u64; let (mut n,mut bad)=(0u64,0u64);
  loop { let mut got=0; while got<buf.len() { let k=rd.read(&mut buf[got..]).unwrap(); if k==0 {break;} got+=k; } if got==0 {break;}
    for rec in buf[..got].chunks_exact(16) { if rec[15]!=0 { let mut k=[0u8;15]; k.copy_from_slice(&rec[..15]); let h=hk(&k);
        if rec[15]==chk(h,rec[15]&1==1) { let mut r=[0u8;16]; r.copy_from_slice(rec); cb(slot,&r); n+=1; } else { if let Some(f)=rw {f.write_at(&[0u8;16],slot*16).unwrap();} bad+=1; } } slot+=1; }
    if got<buf.len(){break;} } assert!(slot==slots,"store size mismatch"); (n,bad) }
// read-only or fp-indexed disk layer (even2 layout)
struct Disk{ f:File, fp:Vec<u8>, mask:u64, n:u64 }
impl Disk{ fn open(p:&str,bits:u32,rw:bool)->Disk{ let slots=1u64<<bits; let f=OpenOptions::new().read(true).write(rw).create(rw).open(p).unwrap();
    let len=f.metadata().unwrap().len(); if len==0 && rw { f.set_len(slots*16).unwrap(); } else { assert!(len==slots*16,"store size mismatch: BITS must match existing file"); }
    let mut fp=vec![0u8;slots as usize]; let t=Instant::now(); let (n,bad)=scan(p,slots,if rw {Some(&f)} else {None},|s,r|{ let mut k=[0u8;15]; k.copy_from_slice(&r[..15]); fp[s as usize]=fpb(hk(&k)); });
    eprintln!("disk layer {} ({}) slots=2^{} loaded {} records ({} invalid {}) in {:.1}s",p,if rw {"rw"} else {"READ-ONLY"},bits,n,bad,if rw {"dropped"} else {"ignored"},t.elapsed().as_secs_f64());
    if !rw { fadv_random(&f); } Disk{f,fp,mask:slots-1,n} }
  fn get(&self,k:&K,fpf:&mut u64)->Option<bool>{ let h=hk(k); let fb=fpb(h); let mut i=h&self.mask;
    loop { let b=self.fp[i as usize]; if b==0 {return None;}
      if b==fb { let mut rec=[0u8;16]; self.f.read_exact_at(&mut rec,i*16).unwrap(); if &rec[..15]==&k[..] { return Some(rec[15]&1==1);} *fpf+=1; }
      i=(i+1)&self.mask; } }
  fn put(&mut self,k:&K,v:bool)->bool{ let h=hk(k); let fb=fpb(h); let mut i=h&self.mask;
    loop { let b=self.fp[i as usize]; if b==0 {break;} if b==fb { let mut rec=[0u8;16]; self.f.read_exact_at(&mut rec,i*16).unwrap(); if &rec[..15]==&k[..] {return false;} } i=(i+1)&self.mask; }
    let mut rec=[0u8;16]; rec[..15].copy_from_slice(k); rec[15]=chk(h,v); self.f.write_all_at(&rec,i*16).unwrap(); self.fp[i as usize]=fb; self.n+=1; true } }
// RAM-indexed writable layer: full records in RAM, file slots tracked by an occupancy bitmap, writes batched & sorted.
struct Ram{ f:File, tbl:Vec<[u8;16]>, rmask:u64, occ:Vec<u64>, fmask:u64, n:u64, pend:Vec<(u64,[u8;16])>, flush:usize,
  tx:Option<std::sync::mpsc::SyncSender<Vec<(u64,[u8;16])>>>, th:Option<std::thread::JoinHandle<()>> }
impl Ram{ fn open(p:&str,bits:u32,rbits:u32,flush:usize)->Ram{ let slots=1u64<<bits; let f=OpenOptions::new().read(true).write(true).create(true).open(p).unwrap();
    let len=f.metadata().unwrap().len(); if len==0 { f.set_len(slots*16).unwrap(); } else { assert!(len==slots*16,"store size mismatch: BITS must match existing file"); }
    let mut r=Ram{f:f.try_clone().unwrap(),tbl:vec![[0u8;16];1usize<<rbits],rmask:(1u64<<rbits)-1,occ:vec![0u64;(slots/64) as usize],fmask:slots-1,n:0,pend:vec![],flush,tx:None,th:None};
    let t=Instant::now(); let cap=(0.85*(r.rmask+1) as f64) as u64; let mut cnt=0u64;
    let (n,bad)=scan(p,slots,Some(&f),|s,rec|{ r.occ[(s>>6) as usize]|=1<<(s&63); cnt+=1; assert!(cnt<cap,"RAMIDX too small for existing records");
      let mut k=[0u8;15]; k.copy_from_slice(&rec[..15]); let mut i=hk(&k)&r.rmask; while r.tbl[i as usize][15]!=0 { i=(i+1)&r.rmask; } r.tbl[i as usize]=*rec; }); r.n=n;
    eprintln!("ram layer {} slots=2^{} ramidx=2^{} loaded {} records ({} invalid dropped) in {:.1}s",p,bits,rbits,n,bad,t.elapsed().as_secs_f64());
    // background writer: batches sorted by slot, written in file order; lookups never wait for it (RAM index is authoritative)
    let (tx,rx)=std::sync::mpsc::sync_channel::<Vec<(u64,[u8;16])>>(1); let wf=f.try_clone().unwrap();
    r.th=Some(std::thread::spawn(move||{ for mut b in rx { b.sort_unstable_by_key(|x|x.0); for (s,rec) in &b { wf.write_all_at(rec,s*16).unwrap(); } } })); r.tx=Some(tx); r }
  fn get(&self,k:&K)->Option<bool>{ let mut i=hk(k)&self.rmask; loop { let e=&self.tbl[i as usize]; if e[15]==0 {return None;} if &e[..15]==&k[..] {return Some(e[15]&1==1);} i=(i+1)&self.rmask; } }
  fn put(&mut self,k:&K,v:bool)->bool{ let h=hk(k); let mut i=h&self.rmask; loop { let e=&self.tbl[i as usize]; if e[15]==0 {break;} if &e[..15]==&k[..] {return false;} i=(i+1)&self.rmask; }
    let mut rec=[0u8;16]; rec[..15].copy_from_slice(k); rec[15]=chk(h,v); self.tbl[i as usize]=rec;
    let mut s=h&self.fmask; while self.occ[(s>>6) as usize]>>(s&63)&1==1 { s=(s+1)&self.fmask; } self.occ[(s>>6) as usize]|=1<<(s&63);
    self.pend.push((s,rec)); self.n+=1; if self.pend.len()>=self.flush {self.do_flush();} true }
  fn do_flush(&mut self){ if self.pend.is_empty(){return;} let b=std::mem::take(&mut self.pend); self.tx.as_ref().unwrap().send(b).unwrap(); }
  fn finish(&mut self){ self.do_flush(); self.tx=None; if let Some(h)=self.th.take(){ h.join().unwrap(); } self.f.sync_data().ok(); } }
struct Store{ ram:Option<Ram>, disk:Option<Disk>, base:Option<Disk>, cap_frac:f64, full:bool, hot:C, old:C, cap:usize,
  st_hit_ram:u64, st_hit_disk:u64, st_hit_base:u64, st_fp_false:u64, st_ins:u64 }
impl Store{
  fn n(&self)->u64{ if let Some(r)=&self.ram {r.n} else if let Some(d)=&self.disk {d.n} else {0} }
  fn slots(&self)->u64{ if let Some(r)=&self.ram {(r.rmask+1).min(r.fmask+1)} else if let Some(d)=&self.disk {d.mask+1} else {1} }
  fn ramget(&self,k:&K)->Option<bool>{ if let Some(&v)=self.hot.get(k){return Some(v);} if let Some(&v)=self.old.get(k){return Some(v);} if let Some(r)=&self.ram { return r.get(k);} None }
  fn get(&mut self,k:&K)->Option<bool>{ if let Some(&v)=self.hot.get(k){self.st_hit_ram+=1; return Some(v);}
    if let Some(&v)=self.old.get(k){ self.st_hit_ram+=1; self.cput(*k,v); return Some(v);}
    if let Some(r)=&self.ram { if let Some(v)=r.get(k){ self.st_hit_ram+=1; return Some(v);} }
    let mut fpf=0; let mut res=None;
    if let Some(d)=&self.disk { if let Some(v)=d.get(k,&mut fpf){ self.st_hit_disk+=1; res=Some(v);} }
    if res.is_none() { if let Some(b)=&self.base { if let Some(v)=b.get(k,&mut fpf){ self.st_hit_base+=1; res=Some(v);} } }
    self.st_fp_false+=fpf; if let Some(v)=res { self.cput(*k,v); } res }
  fn cput(&mut self,k:K,v:bool){ if self.hot.len()>=self.cap { std::mem::swap(&mut self.hot,&mut self.old); self.hot.clear(); } self.hot.insert(k,v); }
  fn put(&mut self,k:K,v:bool,disk:bool){ if !disk || (self.ram.is_none()&&self.disk.is_none()) || self.full { self.cput(k,v); return; }
    if (self.n()+1) as f64 > self.cap_frac*self.slots() as f64 { self.full=true; eprintln!("WARNING store at {:.0}% load: further results kept in RAM cache only (still correct)",self.cap_frac*100.0); self.cput(k,v); return; }
    let ins= if let Some(r)=&mut self.ram { r.put(&k,v) } else { self.cput(k,v); self.disk.as_mut().unwrap().put(&k,v) }; if ins {self.st_ins+=1;} }
  fn close(&mut self){ if let Some(r)=&mut self.ram { r.finish(); } } }
fn rss()->String{ std::fs::read_to_string("/proc/self/status").unwrap_or_default().lines().filter(|l|l.starts_with("VmRSS")||l.starts_with("VmHWM")).map(|l|l.split_whitespace().collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join(" ") }
fn rdbytes()->u64{ std::fs::read_to_string("/proc/self/io").unwrap_or_default().lines().find(|l|l.starts_with("read_bytes")).and_then(|l|l.split_whitespace().last()).and_then(|x|x.parse().ok()).unwrap_or(0) }
struct S{st:Store,em:HashMap<G,u8>,nodes:u64,t0:Instant,next:u64,dmin:u64,killer:Vec<u32>,usek:bool,etc:bool,
  wd_t:f64,wd_n:u64,wd_rb:u64,wd_best:f64,wd_sec:f64,wd_chk:u64,deadline:f64}
fn watchdog(s:&mut S){ let t=s.t0.elapsed().as_secs_f64(); if t-s.wd_t < s.wd_sec {return;} let dn=s.nodes-s.wd_n; let rate=dn as f64/(t-s.wd_t); let rb=rdbytes();
  let mbs=(rb.saturating_sub(s.wd_rb)) as f64/1e6/(t-s.wd_t); if rate>s.wd_best {s.wd_best=rate;}
  eprintln!("[rate] t={:.0}s nodes={} {:.0} nodes/s diskread={:.1} MB/s store={} hit_ram={} hit_disk={} hit_base={} pfail={} {}",t,s.nodes,rate,mbs,s.st.n(),s.st.st_hit_ram,s.st.st_hit_disk,s.st.st_hit_base,unsafe{PFAIL},rss());
  if rate < 0.3*s.wd_best { eprintln!("WATCHDOG WARNING: rate {:.0} nodes/s < 30% of best window {:.0} nodes/s (I/O thrash or contention?)",rate,s.wd_best); }
  s.wd_t=t; s.wd_n=s.nodes; s.wd_rb=rb; }
fn win(s:&mut S,w:&W)->bool{ let M=md(); let f=*w[..M].iter().max().unwrap() as usize-M; if f<=1 {return false;} let k=key(w);
  if let Some(v)=s.st.get(&k){return v;} s.nodes+=1;
  if s.nodes>=s.next { s.next+=50_000_000; let st=&s.st; eprintln!("[prog] nodes={} store={} ram_hit={} disk_hit={} base_hit={} fp_false={} {} {:.0}s",s.nodes,st.n(),st.st_hit_ram,st.st_hit_disk,st.st_hit_base,st.st_fp_false,rss(),s.t0.elapsed().as_secs_f64()); }
  if s.nodes>=s.wd_chk { s.wd_chk=s.nodes+200_000; watchdog(s);
    if s.deadline>0.0 && s.t0.elapsed().as_secs_f64()>s.deadline { s.st.close(); eprintln!("[deadline] {}s reached after {} nodes: store flushed (store={}), exiting TIMEOUT",s.deadline,s.nodes,s.st.n()); println!("TIMEOUT nodes={}",s.nodes); std::process::exit(4); } }
  let n0=s.nodes; let mut r=false;
  if s.etc { let mut x=f; while x>=2 { if x<w[x%M] as usize { let c=wadd(w,x); let cf=*c[..M].iter().max().unwrap() as usize-M;
        if cf<=1 || s.st.ramget(&key(&c))==Some(false) { r=true; break; } } x-=1; } }
  if !r { let kx= if s.usek {s.killer[f] as usize} else {0};
    if kx>=2 && kx<=f && kx<w[kx%M] as usize { if !win(s,&wadd(w,kx)){r=true;} }
    if !r { let mut x=f; while x>=2 { if x!=kx && x<w[x%M] as usize { if !win(s,&wadd(w,x)){r=true; if s.usek {s.killer[f]=x as u32;} break;} } x-=1; } } }
  let disk=s.nodes-n0>=s.dmin; s.st.put(k,r,disk); r }
fn quiet(g:&G)->bool{ let t=frob(g); if t%2==0 {return false;} (1..t).all(|m| get(g,m)!=get(g,t-m)) }
fn odd_pos(sg:&G,n:usize)->G{ let t=frob(sg); let lim=2*t+2*n+6; let mut inn=vec![false;lim]; for i in 0..lim/2 { if !get(sg,i){inn[2*i]=true;} }
  for k in n..lim { if inn[k-n]{inn[k]=true;} } from_in(&inn) }
fn winp(s:&mut S,sg:&G,n:usize)->bool{ let w=g2w(&odd_pos(sg,n)); win(s,&w) }
#[allow(non_snake_case)]
fn mingens(g:&G)->Vec<usize>{ let t=frob(g); let mut inn=vec![false;t+2+2*(t+2)]; let L=inn.len();
  for i in 0..L { inn[i]= i==0 || !get(g,i); } let mut out=vec![];
  for x in 1..L { if !inn[x]{continue;} let mut red=false; for y in 1..x { if inn[y]&&inn[x-y]{red=true;break;} } if !red {out.push(x);} if out.len()>0 && x> t+1+out[0] {break;} } out }
fn lemma_p(t:&[usize])->bool{ let l:[&[usize];7]=[&[4,6],&[8,14],&[8,10,22],&[8,20,30],&[8,20,26],&[8,18,28],&[8,34,36]];
  if l.iter().any(|x| *x==t) {return true;}
  if t.contains(&8) && t.contains(&12) { let r:Vec<usize>=t.iter().cloned().filter(|&x|x!=8&&x!=12).collect(); if r.is_empty(){return true;} if r.len()==2 { return r[0]%8==2 && r[1]==r[0]+4;} }
  false }
fn pattern_first(sg:&G)->bool{ let tg:Vec<String>=mingens(sg).iter().map(|x|(2*x).to_string()).collect(); let key=tg.join(",");
  let lim=std::env::var("PLIM").unwrap_or("100001".into());
  if let Ok(c)=std::fs::read_to_string("/workspace/sylver/pattern_cache.tsv") { for l in c.lines(){ let f:Vec<&str>=l.split('\t').collect();
    if f.len()==3 && f[0]==key { if f[1]!="none" {return true;} if f[2].parse::<u64>().unwrap_or(0)>=lim.parse::<u64>().unwrap() {return false;} } } }
  let pul=std::env::var("PULIM").unwrap_or("3000000".into());
  let out=std::process::Command::new("sh").arg("-c").arg(format!("ulimit -Sv {}; exec /workspace/sylver/even2/pattern2 first {} {}",pul,lim,tg.join(" "))).output().unwrap();
  if !out.status.success() { unsafe{PFAIL+=1;} eprintln!("PATTERN SOLVER FAILED for {} ({:?}) -> this branch counts as UNKNOWN; pfail={} stderr: {}",key,out.status,unsafe{PFAIL},String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or(""));
    if std::env::var("PFAIL_HARD").map(|x|x=="1").unwrap_or(false) { eprintln!("PFAIL_HARD=1: aborting"); std::process::exit(3); } return false; }
  let o=String::from_utf8_lossy(&out.stdout).to_string(); let w=o.lines().find(|l|l.starts_with("odd winning move")).map(|l|l.split(' ').last().unwrap().to_string());
  let mut f=std::fs::OpenOptions::new().create(true).append(true).open("/workspace/sylver/pattern_cache.tsv").unwrap();
  writeln!(f,"{}\t{}\t{}",key,w.clone().unwrap_or("none".into()),lim).unwrap(); if std::env::var("DBG").is_ok(){eprintln!("pattern {} -> {:?}",key,w);} w.is_some() }
fn val2(s:&mut S,sg:&G)->u8{ let t=frob(sg); if t==0 {return 1;} if let Some(&v)=s.em.get(sg){return v;}
  { let tg:Vec<usize>=mingens(sg).iter().map(|x|2*x).collect(); if lemma_p(&tg){ s.em.insert(sg.clone(),0); return 0; } }
  let q=quiet(sg); let mut unk=!q; let mut r=0u8;
  for m in (1..=t).rev(){ if !get(sg,m){continue;}
    if m%2==1 && m>1 { if !winp(s,sg,m){r=1; break;} }
    let c=val2(s,&add(sg,m)); if c==0 {r=1; break;} if c==2 {unk=true;} }
  if r==0 && !q { let b:usize=env::var("ODDB").ok().and_then(|x|x.parse().ok()).unwrap_or(4*t+40);
    let mut n=3; while n<=b { if !get(sg,n){ if !winp(s,sg,n){r=1; break;} } n+=2; } }
  if r==0 && !q { if pattern_first(sg) {r=1;} }
  if r==0 && unk {r=2;} s.em.insert(sg.clone(),r); r }
fn selftest(){ let mut seed=12345u64; let mut rnd=||{ seed^=seed<<13; seed^=seed>>7; seed^=seed<<17; seed };
  let mut nt=0; for m in [16usize,8,10,12,14,6,4] { unsafe{MM=m;} for _ in 0..20000 { let mut w=[0u32;MX]; for r in 1..m { w[r]=(r+m*(1+(rnd()%12) as usize)) as u32; }
    let w=wadd_old(&w,m*100+1).map(|x|x); // normalise to a valid Apery set
    for _ in 0..4 { let a=2+(rnd()%300) as usize; if a%m==0 || a as u32>=w[a%m] {continue;} let o=wadd_old(&w,a); let n=wadd(&w,a); assert!(o[..m]==n[..m],"wadd mismatch m={} a={} w={:?}",m,a,&w[..m]); nt+=1; } } }
  println!("selftest wadd: {} random cases identical to the O(M^2) version",nt); }
fn main(){ let args:Vec<String>=env::args().skip(1).collect(); if args.len()==1 && args[0]=="selftest" { selftest(); return; }
  let v:Vec<usize>=args.iter().map(|x|x.parse::<usize>().unwrap()/2).collect(); let t0=Instant::now();
  let ev=|n:&str,d:u64|->u64{ env::var(n).ok().and_then(|x|x.parse().ok()).unwrap_or(d) };
  let bits=ev("BITS",28) as u32; let cap=ev("CACHE",14_000_000) as usize; let rbits=ev("RAMIDX",0) as u32;
  let (mut ram,mut disk)=(None,None);
  if let Ok(p)=env::var("STORE") { if rbits>0 { ram=Some(Ram::open(&p,bits,rbits,ev("FLUSH",1_000_000) as usize)); } else { disk=Some(Disk::open(&p,bits,true)); } }
  let base=env::var("BASE").ok().map(|p| Disk::open(&p,ev("BASE_BITS",29) as u32,false));
  let st=Store{ram,disk,base,cap_frac:0.85,full:false,hot:C::with_capacity_and_hasher(cap,Default::default()),old:C::with_capacity_and_hasher(cap,Default::default()),cap,
    st_hit_ram:0,st_hit_disk:0,st_hit_base:0,st_fp_false:0,st_ins:0};
  let n0=st.n(); let dmin=ev("DMIN",1);
  let mut s=S{st,em:HashMap::new(),nodes:0,t0,next:50_000_000,dmin,killer:vec![0u32;4096],usek:ev("KILLER",0)==1,etc:ev("ETC",0)==1,
    wd_t:0.0,wd_n:0,wd_rb:rdbytes(),wd_best:0.0,wd_sec:ev("RATE_SEC",300) as f64,wd_chk:200_000,deadline:ev("DEADLINE",0) as f64};
  unsafe{ MM=2**v.iter().min().unwrap(); } assert!(md()>=2 && md()<=16,"min generator must be <=16"); eprintln!("modulus M={} killer={} etc={}",md(),s.usek,s.etc);
  if s.st.base.is_some() { assert!(md()==16,"BASE layer only valid for M=16 key space"); }
  let sg=gens(&v); let r=val2(&mut s,&sg); s.st.close();
  eprintln!("[final] nodes={} store={} (new {}) ram_hit={} disk_hit={} base_hit={} fp_false={} pfail={} {}",s.nodes,s.st.n(),s.st.n()-n0,s.st.st_hit_ram,s.st.st_hit_disk,s.st.st_hit_base,s.st.st_fp_false,unsafe{PFAIL},rss());
  println!("{:?} S=T/2 quiet={} {} nodes={} {:.2}s even3 pfail={}",v.iter().map(|x|x*2).collect::<Vec<_>>(),quiet(&sg),["P","N","UNKNOWN"][r as usize],s.nodes,t0.elapsed().as_secs_f64(),unsafe{PFAIL}); }
