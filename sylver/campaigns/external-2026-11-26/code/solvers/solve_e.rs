// Sylver coinage solver for gcd=1 positions. Position = gap set (bitset) of numerical semigroup.
// win(g)=true: player to move wins (N). Gap set {1} only => mover must name 1 => P.
use std::collections::HashMap; use std::env; use std::time::Instant;
type G=Vec<u64>;
fn get(g:&G,i:usize)->bool{ g[i>>6]>>(i&63)&1==1 }
fn frob(g:&G)->usize{ for w in (0..g.len()).rev(){ if g[w]!=0 {return w*64+63-g[w].leading_zeros() as usize;} } 0 }
fn add(g:&G,a:usize)->G{ let f=frob(g); let mut inn=vec![true;f+1];
  for i in 0..=f { if get(g,i){inn[i]=false;} } inn[a]=true;
  for n in a..=f { if inn[n-a]{inn[n]=true;} }
  let mut r=vec![0u64;f/64+1]; let mut nf=0; for i in 0..=f { if !inn[i]{ r[i>>6]|=1<<(i&63); nf=i; } }
  r.truncate(nf/64+1); r }
struct S{memo:HashMap<G,bool>,nodes:u64,cap:usize,enders:u64}
// Weak Antisymmetry: S is an ender iff no m<n, m+n=t, with both m and n gaps (Sicherman, Integers 2 (2002) G2). Enders with t>1 are N.
fn ender(g:&G,t:usize)->bool{ let mut m=1; while 2*m<t { if get(g,m)&&get(g,t-m){return false;} m+=1; } true }
fn win(s:&mut S,g:&G)->bool{ let f=frob(g); if f<=1 {return false;}
  if let Some(&v)=s.memo.get(g){return v;} if ender(g,f) {s.enders+=1; return true;} s.nodes+=1;
  let mut r=false; let mut x=f; while x>=2 { if get(g,x){ let h=add(g,x); if !win(s,&h){r=true;break;} } x-=1; }
  if s.memo.len()<s.cap { s.memo.insert(g.clone(),r);} r }
fn gens(v:&[usize])->G{ let m=*v.iter().min().unwrap(); let lim=v.iter().max().unwrap()*m+5;
  let mut inn=vec![false;lim]; inn[0]=true; for &a in v { for n in a..lim { if inn[n-a]{inn[n]=true;} } }
  let mut r=vec![0u64;lim/64+1]; for i in 0..lim { if !inn[i]{ r[i>>6]|=1<<(i&63);} } let f=frob(&r); r.truncate(f/64+1); r }
fn main(){ let v:Vec<usize>=env::args().skip(1).map(|x|x.parse().unwrap()).collect();
  let cap:usize=env::var("CAP").ok().and_then(|x|x.parse().ok()).unwrap_or(100_000_000);
  let g=gens(&v); let t=Instant::now(); let mut s=S{memo:HashMap::new(),nodes:0,cap,enders:0};
  let r=win(&mut s,&g);
  println!("{:?} F={} {} nodes={} memo={} enders={} {:.2}s",v,frob(&g),if r{"N"}else{"P"},s.nodes,s.memo.len(),s.enders,t.elapsed().as_secs_f64()); }
