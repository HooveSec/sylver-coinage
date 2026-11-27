// Sylver coinage solver for positions containing modulus M (default 16) with gcd 1.
// State = Apery set w[r] = least element of semigroup ≡ r mod M (Böcker–Lipták round-robin update).
// x is a legal move iff x < w[x mod M]. Mover with only gap {1} loses (P).
use std::collections::HashMap; use std::hash::{BuildHasherDefault,Hasher}; use std::env; use std::time::Instant;
const M:usize=16; type W=[u32;M];
#[derive(Default)] struct Fx(u64);
impl Hasher for Fx{ fn finish(&self)->u64{self.0} fn write(&mut self,b:&[u8]){for &x in b{self.0=(self.0.rotate_left(5)^x as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);}}
  fn write_u32(&mut self,x:u32){self.0=(self.0.rotate_left(5)^x as u64).wrapping_mul(0x517cc1b727220a95);} }
type H=HashMap<W,bool,BuildHasherDefault<Fx>>;
fn add(w:&W,a:usize)->W{ let mut v=*w; let ar=a%M; // round-robin relaxation along cycles of +a
  if (a as u32)<v[ar]{v[ar]=a as u32;}
  for _ in 0..2 { for s in 0..M { let mut r=s; for _ in 0..M { let n=(r+ar)%M; let c=v[r]+a as u32; if c<v[n]{v[n]=c;} r=n; } } } v }
fn terminal(w:&W)->bool{ (2..M).all(|r|w[r] as usize==r) && w[1] as usize<=M+1 }
fn win(h:&mut H,w:&W,nodes:&mut u64,cap:usize)->bool{ if terminal(w){return false;}
  if let Some(&v)=h.get(w){return v;} *nodes+=1;
  let f=*w.iter().max().unwrap() as usize - M; let mut r=false; let mut x=f;
  while x>=2 { if x < w[x%M] as usize { if !win(h,&add(w,x),nodes,cap){r=true;break;} } x-=1; }
  if h.len()<cap {h.insert(*w,r);} r }
fn main(){ let v:Vec<usize>=env::args().skip(1).map(|x|x.parse().unwrap()).collect();
  let cap:usize=env::var("CAP").ok().and_then(|x|x.parse().ok()).unwrap_or(250_000_000);
  let mut w=[u32::MAX/2;M]; w[0]=0; for &a in &v { w=add(&w,a); }
  let t=Instant::now(); let mut h=H::default(); let mut n=0; let r=win(&mut h,&w,&mut n,cap);
  println!("[16, {}] F={} {} nodes={} memo={} {:.2}s apery",v.iter().map(|x|x.to_string()).collect::<Vec<_>>().join(", "),*w.iter().max().unwrap() as i64-M as i64,if r{"N"}else{"P"},n,h.len(),t.elapsed().as_secs_f64()); }
