// even2: g=2 Sylver coinage solver (same game logic as even.rs) with the finite-game memo moved to disk.
// Finite positions (all contain 16) are keyed by Kunz coordinates k[r]=(w[r]-r)/16, r=1..15, of the Apery set mod 16
// (15 bytes = FULL key, stored on disk so collisions are always checked exactly).
// Store: STORE=<path> file of 2^BITS 16-byte records (open addressing, linear probing), write-through via pwrite;
// RAM: 1-byte fingerprint per slot + 2-generation bounded cache (pre-reserved, never grows -> no doubling).
// Only exact finite-game values (win/lose, fully evaluated) are stored. g=2 results (P/N/UNKNOWN) stay in RAM only.
use std::collections::HashMap; use std::env; use std::time::Instant; use std::fs::{File,OpenOptions};
use std::os::unix::fs::FileExt; use std::io::{Read,Write}; use std::hash::{BuildHasherDefault,Hasher};
type G=Vec<u64>; const MX:usize=16; type W=[u32;MX]; type K=[u8;15];
// Modulus M = smallest generator of the starting position (always an element of every descendant position).
static mut MM:usize=16; fn md()->usize{ unsafe{MM} }
fn get(g:&G,i:usize)->bool{ i<g.len()*64 && g[i>>6]>>(i&63)&1==1 }
fn frob(g:&G)->usize{ for w in (0..g.len()).rev(){ if g[w]!=0 {return w*64+63-g[w].leading_zeros() as usize;} } 0 }
fn from_in(inn:&[bool])->G{ let mut f=0; for i in 1..inn.len(){ if !inn[i]{f=i;} } let mut r=vec![0u64;f/64+1]; for i in 1..=f { if !inn[i]{ r[i>>6]|=1<<(i&63);} } r }
fn add(g:&G,a:usize)->G{ let f=frob(g); let mut inn=vec![true;f+1]; for i in 0..=f { if get(g,i){inn[i]=false;} }
  if a<=f {inn[a]=true;} for n in a..=f { if inn[n-a]{inn[n]=true;} } from_in(&inn) }
fn gens(v:&[usize])->G{ let m=*v.iter().min().unwrap(); let lim=v.iter().max().unwrap()*m+5; let mut inn=vec![false;lim]; inn[0]=true;
  for &a in v { for n in a..lim { if inn[n-a]{inn[n]=true;} } } from_in(&inn) }
// ---------- Apery-set finite game ----------
fn wadd(w:&W,a:usize)->W{ let M=md(); let mut v=*w; let ar=a%M; if (a as u32)<v[ar]{v[ar]=a as u32;}
  for s in 0..M { let mut r=s; for _ in 0..M { let n=(r+ar)%M; let c=v[r]+a as u32; if c<v[n]{v[n]=c;} r=n; } } v }
fn g2w(g:&G)->W{ let M=md(); assert!(!get(g,M),"modulus not in position"); let f=frob(g); let mut w=[0u32;MX]; for r in 0..M {w[r]=u32::MAX;} for x in 0..=f+M { let r=x%M; if w[r]==u32::MAX && !get(g,x){w[r]=x as u32;} }
  assert!(w[0]==0 && w[..M].iter().all(|&x|x!=u32::MAX),"bad apery"); w }
fn key(w:&W)->K{ let M=md(); let mut k=[0u8;15]; for r in 1..M { let q=(w[r] as usize-r)/M; assert!(q<240,"Kunz coordinate overflow"); k[r-1]=q as u8; } if M<16 {k[14]=0xF0+M as u8;} k }
fn hk(k:&K)->u64{ let mut a=[0u8;16]; a[..15].copy_from_slice(k); let x=u64::from_le_bytes(a[..8].try_into().unwrap()); let y=u64::from_le_bytes(a[8..].try_into().unwrap());
  let mut h=x.wrapping_mul(0x9E3779B97F4A7C15)^y.rotate_left(29).wrapping_mul(0xC2B2AE3D27D4EB4F); h^=h>>31; h=h.wrapping_mul(0xFF51AFD7ED558CCD); h^=h>>33; h=h.wrapping_mul(0xC4CEB9FE1A85EC53); h^h>>29 }
#[derive(Default)] struct Id(u64); impl Hasher for Id{ fn finish(&self)->u64{self.0} fn write(&mut self,b:&[u8]){ let mut k=[0u8;15]; if b.len()==15 {k.copy_from_slice(b); self.0=hk(&k);} else { for &x in b {self.0=(self.0.rotate_left(8)^x as u64).wrapping_mul(0x9E3779B97F4A7C15);} } } }
type C=HashMap<K,bool,BuildHasherDefault<Id>>;
struct Store{ f:Option<File>, fp:Vec<u8>, mask:u64, n:u64, full:bool, hot:C, old:C, cap:usize,
  st_hit_ram:u64, st_hit_disk:u64, st_fp_false:u64, st_ins:u64 }
fn chk(h:u64,v:bool)->u8{ 0x80 | (((h>>40) as u8)&0x7e) | v as u8 }
fn fpb(h:u64)->u8{ let b=(h>>56) as u8; if b==0 {1} else {b} }
impl Store{
  fn open(path:Option<String>,bits:u32,cap:usize)->Store{ let mut s=Store{f:None,fp:vec![],mask:0,n:0,full:false,
      hot:C::with_capacity_and_hasher(cap,Default::default()),old:C::with_capacity_and_hasher(cap,Default::default()),cap,st_hit_ram:0,st_hit_disk:0,st_fp_false:0,st_ins:0};
    if let Some(p)=path { let slots=1u64<<bits; s.mask=slots-1; s.fp=vec![0u8;slots as usize];
      let f=OpenOptions::new().read(true).write(true).create(true).open(&p).unwrap(); let len=f.metadata().unwrap().len();
      if len==0 { f.set_len(slots*16).unwrap(); } else { assert!(len==slots*16,"store size mismatch: BITS must match existing file"); }
      // rebuild fingerprints by a sequential scan; drop torn/invalid records
      let t=Instant::now(); let mut rd=File::open(&p).unwrap(); let mut buf=vec![0u8;1<<24]; let mut slot=0u64; let mut bad=0u64;
      loop { let mut got=0; while got<buf.len() { let k=rd.read(&mut buf[got..]).unwrap(); if k==0 {break;} got+=k; } if got==0 {break;}
        for rec in buf[..got].chunks_exact(16) { if rec[15]!=0 { let mut k=[0u8;15]; k.copy_from_slice(&rec[..15]); let h=hk(&k);
            if rec[15]==chk(h,rec[15]&1==1) { s.fp[slot as usize]=fpb(h); s.n+=1; } else { f.write_at(&[0u8;16],slot*16).unwrap(); bad+=1; } } slot+=1; }
        if got<buf.len(){break;} }
      eprintln!("store {} slots=2^{} loaded {} records ({} invalid dropped) in {:.1}s",p,bits,s.n,bad,t.elapsed().as_secs_f64()); s.f=Some(f); }
    s }
  fn get(&mut self,k:&K)->Option<bool>{ if let Some(&v)=self.hot.get(k){self.st_hit_ram+=1; return Some(v);}
    if let Some(&v)=self.old.get(k){ self.st_hit_ram+=1; self.cput(*k,v); return Some(v);}
    let f=match &self.f {Some(f)=>f,None=>return None}; let h=hk(k); let fb=fpb(h); let mut i=h&self.mask;
    loop { let b=self.fp[i as usize]; if b==0 {return None;}
      if b==fb { let mut rec=[0u8;16]; f.read_exact_at(&mut rec,i*16).unwrap(); if &rec[..15]==&k[..] { let v=rec[15]&1==1; self.st_hit_disk+=1; self.cput(*k,v); return Some(v);} self.st_fp_false+=1; }
      i=(i+1)&self.mask; } }
  fn cput(&mut self,k:K,v:bool){ if self.hot.len()>=self.cap { std::mem::swap(&mut self.hot,&mut self.old); self.hot.clear(); } self.hot.insert(k,v); }
  fn put(&mut self,k:K,v:bool,disk:bool){ self.cput(k,v); if self.f.is_none()||self.full||!disk {return;}
    if (self.n+1) as f64 > 0.85*(self.mask+1) as f64 { self.full=true; eprintln!("WARNING store at 85% load: further results kept in RAM cache only (still correct)"); return; }
    let h=hk(&k); let fb=fpb(h); let mut i=h&self.mask; let f=self.f.as_ref().unwrap();
    loop { let b=self.fp[i as usize]; if b==0 {break;} if b==fb { let mut rec=[0u8;16]; f.read_exact_at(&mut rec,i*16).unwrap(); if &rec[..15]==&k[..] {return;} } i=(i+1)&self.mask; }
    let mut rec=[0u8;16]; rec[..15].copy_from_slice(&k); rec[15]=chk(h,v); f.write_all_at(&rec,i*16).unwrap(); self.fp[i as usize]=fb; self.n+=1; self.st_ins+=1; } }
fn rss()->String{ std::fs::read_to_string("/proc/self/status").unwrap_or_default().lines().filter(|l|l.starts_with("VmRSS")||l.starts_with("VmHWM")).map(|l|l.split_whitespace().collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join(" ") }
struct S{st:Store,em:HashMap<G,u8>,nodes:u64,t0:Instant,next:u64,dmin:u64}
fn win(s:&mut S,w:&W)->bool{ let M=md(); let f=*w[..M].iter().max().unwrap() as usize-M; if f<=1 {return false;} let k=key(w);
  if let Some(v)=s.st.get(&k){return v;} s.nodes+=1;
  if s.nodes>=s.next { s.next+=50_000_000; let st=&s.st; eprintln!("[prog] nodes={} store={} ram_hit={} disk_hit={} fp_false={} {} {:.0}s",s.nodes,st.n,st.st_hit_ram,st.st_hit_disk,st.st_fp_false,rss(),s.t0.elapsed().as_secs_f64()); }
  let n0=s.nodes; let mut r=false; let mut x=f; while x>=2 { if x<w[x%M] as usize { if !win(s,&wadd(w,x)){r=true;break;} } x-=1; }
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
  let pul=std::env::var("PULIM").unwrap_or("3000000".into()); // pattern child gets its own soft address-space cap (KB)
  let out=std::process::Command::new("sh").arg("-c").arg(format!("ulimit -Sv {}; exec /workspace/sylver/even2/pattern2 first {} {}",pul,lim,tg.join(" "))).output().unwrap();
  if !out.status.success() { eprintln!("pattern solver failed for {} ({:?})",key,out.status); return false; }
  let o=String::from_utf8_lossy(&out.stdout).to_string(); let w=o.lines().find(|l|l.starts_with("odd winning move")).map(|l|l.split(' ').last().unwrap().to_string());
  let mut f=std::fs::OpenOptions::new().create(true).append(true).open("/workspace/sylver/pattern_cache.tsv").unwrap();
  writeln!(f,"{}\t{}\t{}",key,w.clone().unwrap_or("none".into()),lim).unwrap(); if std::env::var("DBG").is_ok(){eprintln!("pattern {} -> {:?}",key,w);} w.is_some() }
fn val2(s:&mut S,sg:&G)->u8{ let t=frob(sg); if t==0 {return 1;} if let Some(&v)=s.em.get(sg){return v;}
  { let tg:Vec<usize>=mingens(sg).iter().map(|x|2*x).collect(); if lemma_p(&tg){ s.em.insert(sg.clone(),0); return 0; } }
  let q=quiet(sg); let mut unk=!q; let mut r=0u8;
  for m in (1..=t).rev(){ if !get(sg,m){continue;}
    if m%2==1 && m>1 { if !winp(s,sg,m){r=1; if env::var("DBG").is_ok(){eprintln!("win odd {} in S gaps {:?}",m,(1..=t).filter(|&i|get(sg,i)).collect::<Vec<_>>());} break;} }
    let c=val2(s,&add(sg,m)); if c==0 {r=1; if env::var("DBG").is_ok(){eprintln!("win even {} in S gaps {:?}",2*m,(1..=t).filter(|&i|get(sg,i)).collect::<Vec<_>>());} break;} if c==2 {unk=true;} }
  if r==0 && !q { let b:usize=env::var("ODDB").ok().and_then(|x|x.parse().ok()).unwrap_or(4*t+40);
    let mut n=3; while n<=b { if !get(sg,n){ if !winp(s,sg,n){r=1; if env::var("DBG").is_ok(){eprintln!("win big-odd {}",n);} break;} } n+=2; } }
  if r==0 && !q { if pattern_first(sg) {r=1;} }
  if r==0 && unk {r=2;} s.em.insert(sg.clone(),r); r }
fn main(){ let v:Vec<usize>=env::args().skip(1).map(|x|x.parse::<usize>().unwrap()/2).collect(); let t0=Instant::now();
  let bits:u32=env::var("BITS").ok().and_then(|x|x.parse().ok()).unwrap_or(28); let cap:usize=env::var("CACHE").ok().and_then(|x|x.parse().ok()).unwrap_or(14_000_000);
  let st=Store::open(env::var("STORE").ok(),bits,cap); let n0=st.n;
  let dmin:u64=env::var("DMIN").ok().and_then(|x|x.parse().ok()).unwrap_or(1); // only subtrees with >=DMIN evaluated nodes go to disk
  let mut s=S{st,em:HashMap::new(),nodes:0,t0,next:50_000_000,dmin}; unsafe{ MM=2**v.iter().min().unwrap(); } assert!(md()>=2 && md()<=16,"min generator must be <=16"); eprintln!("modulus M={}",md());
  let sg=gens(&v); let r=val2(&mut s,&sg);
  eprintln!("[final] nodes={} store={} (new {}) ram_hit={} disk_hit={} fp_false={} {}",s.nodes,s.st.n,s.st.n-n0,s.st.st_hit_ram,s.st.st_hit_disk,s.st.st_fp_false,rss());
  println!("{:?} S=T/2 quiet={} {} nodes={} {:.2}s even2",v.iter().map(|x|x*2).collect::<Vec<_>>(),quiet(&sg),["P","N","UNKNOWN"][r as usize],s.nodes,t0.elapsed().as_secs_f64()); }
