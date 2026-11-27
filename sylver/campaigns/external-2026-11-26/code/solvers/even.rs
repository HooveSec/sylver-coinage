// g=2 Sylver coinage solver using Sicherman's Quiet End Theorem.
// Position T = 2*S with gcd(S)=1. Moves: even 2m (m gap of S) -> T'=2*(S+m), recurse;
// odd n -> finite position <2S,n>. If S is a quiet ender, every odd n ILLEGAL in S yields a quiet ender (N),
// so only odd n that are gaps of S need search (finite). If S is not a quiet ender ("long"), result may be UNKNOWN.
use std::collections::HashMap; use std::env; use std::time::Instant;
type G=Vec<u64>;
fn get(g:&G,i:usize)->bool{ i<g.len()*64 && g[i>>6]>>(i&63)&1==1 }
fn frob(g:&G)->usize{ for w in (0..g.len()).rev(){ if g[w]!=0 {return w*64+63-g[w].leading_zeros() as usize;} } 0 }
fn from_in(inn:&[bool])->G{ let mut f=0; for i in 1..inn.len(){ if !inn[i]{f=i;} } let mut r=vec![0u64;f/64+1]; for i in 1..=f { if !inn[i]{ r[i>>6]|=1<<(i&63);} } r }
fn add(g:&G,a:usize)->G{ let f=frob(g); let mut inn=vec![true;f+1]; for i in 0..=f { if get(g,i){inn[i]=false;} }
  if a<=f {inn[a]=true;} for n in a..=f { if inn[n-a]{inn[n]=true;} } from_in(&inn) }
fn gens(v:&[usize])->G{ let m=*v.iter().min().unwrap(); let lim=v.iter().max().unwrap()*m+5; let mut inn=vec![false;lim]; inn[0]=true;
  for &a in v { for n in a..lim { if inn[n-a]{inn[n]=true;} } } from_in(&inn) }
struct S{fm:HashMap<G,bool>,em:HashMap<G,u8>,nodes:u64}
fn win(s:&mut S,g:&G)->bool{ let f=frob(g); if f<=1 {return false;} if let Some(&v)=s.fm.get(g){return v;} s.nodes+=1;
  let mut r=false; let mut x=f; while x>=2 { if get(g,x){ if !win(s,&add(g,x)){r=true;break;} } x-=1; } s.fm.insert(g.clone(),r); r }
fn quiet(g:&G)->bool{ let t=frob(g); if t%2==0 {return false;} (1..t).all(|m| get(g,m)!=get(g,t-m)) }
fn odd_pos(sg:&G,n:usize)->G{ let t=frob(sg); let lim=2*t+2*n+6; let mut inn=vec![false;lim]; for i in 0..lim/2 { if !get(sg,i){inn[2*i]=true;} }
  for k in n..lim { if inn[k-n]{inn[k]=true;} } from_in(&inn) }
fn mingens(g:&G)->Vec<usize>{ let t=frob(g); let mut inn=vec![false;t+2+2*(t+2)]; let L=inn.len();
  for i in 0..L { inn[i]= i==0 || !get(g,i); } let mut out=vec![];
  for x in 1..L { if !inn[x]{continue;} let mut red=false; for y in 1..x { if inn[y]&&inn[x-y]{red=true;break;} } if !red {out.push(x);} if out.len()>0 && x> t+1+out[0] {break;} } out }
// Published P-positions used as lemmas (T = 2S, generators). Sources: Blok, "Sylver Coinage positions with g=2" (2021) tables for {8};
// Sicherman enders page ({4,6}); pairing strategy {8,12},{8,12,8a+2,8b+6}.
fn lemma_p(t:&[usize])->bool{ let l:[&[usize];7]=[&[4,6],&[8,14],&[8,10,22],&[8,20,30],&[8,20,26],&[8,18,28],&[8,34,36]];
  if l.iter().any(|x| *x==t) {return true;}
  if t.contains(&8) && t.contains(&12) { let r:Vec<usize>=t.iter().cloned().filter(|&x|x!=8&&x!=12).collect(); if r.is_empty(){return true;} if r.len()==2 { return r[0]%8==2 && r[1]==r[0]+4;} }
  false }
// Disk-memoized call to the pattern solver: does long position 2S have an odd winning move <= PLIM?
fn pattern_first(sg:&G)->bool{ let tg:Vec<String>=mingens(sg).iter().map(|x|(2*x).to_string()).collect(); let key=tg.join(",");
  let lim=std::env::var("PLIM").unwrap_or("100001".into());
  if let Ok(c)=std::fs::read_to_string("/workspace/sylver/pattern_cache.tsv") { for l in c.lines(){ let f:Vec<&str>=l.split('\t').collect();
    if f.len()==3 && f[0]==key { if f[1]!="none" {return true;} if f[2].parse::<u64>().unwrap_or(0)>=lim.parse::<u64>().unwrap() {return false;} } } }
  let out=std::process::Command::new("/workspace/sylver/pattern").arg("first").arg(&lim).args(&tg).output().unwrap();
  if !out.status.success() { eprintln!("pattern solver failed for {} ({:?})",key,out.status); return false; }
  let o=String::from_utf8_lossy(&out.stdout).to_string(); let w=o.lines().find(|l|l.starts_with("odd winning move")).map(|l|l.split(' ').last().unwrap().to_string());
  use std::io::Write; let mut f=std::fs::OpenOptions::new().create(true).append(true).open("/workspace/sylver/pattern_cache.tsv").unwrap();
  writeln!(f,"{}\t{}\t{}",key,w.clone().unwrap_or("none".into()),lim).unwrap(); if std::env::var("DBG").is_ok(){eprintln!("pattern {} -> {:?}",key,w);} w.is_some() }
// 0=P 1=N 2=UNKNOWN
fn val2(s:&mut S,sg:&G)->u8{ let t=frob(sg); if t==0 {return 1;} if let Some(&v)=s.em.get(sg){return v;}
  { let tg:Vec<usize>=mingens(sg).iter().map(|x|2*x).collect(); if lemma_p(&tg){ s.em.insert(sg.clone(),0); return 0; } }
  let q=quiet(sg); let mut unk=!q; let mut r=0u8;
  for m in (1..=t).rev(){ if !get(sg,m){continue;}
    if m%2==1 && m>1 { if !win(s,&odd_pos(sg,m)){r=1; if std::env::var("DBG").is_ok(){eprintln!("win odd {} in S gaps {:?}",m,(1..=t).filter(|&i|get(sg,i)).collect::<Vec<_>>());} break;} }
    let c=val2(s,&add(sg,m)); if c==0 {r=1; if std::env::var("DBG").is_ok(){eprintln!("win even {} in S gaps {:?}",2*m,(1..=t).filter(|&i|get(sg,i)).collect::<Vec<_>>());} break;} if c==2 {unk=true; if std::env::var("DBG").is_ok(){eprintln!("unknown child: S+{} gaps={:?}",m,(1..=frob(&add(sg,m))).filter(|&i|get(&add(sg,m),i)).collect::<Vec<_>>());}} }
  if r==0 && !q { // long: try odd moves illegal in S (finite games), up to bound; a P child proves N
    let b:usize=std::env::var("ODDB").ok().and_then(|x|x.parse().ok()).unwrap_or(4*t+40);
    let mut n=3; while n<=b { if !get(sg,n){ if !win(s,&odd_pos(sg,n)){r=1; if std::env::var("DBG").is_ok(){eprintln!("win big-odd {}",n);} break;} } n+=2; } }
  if r==0 && !q { if pattern_first(sg) {r=1;} }
  if r==0 && unk {r=2;} s.em.insert(sg.clone(),r); r }
fn main(){ let v:Vec<usize>=env::args().skip(1).map(|x|x.parse::<usize>().unwrap()/2).collect(); let t0=Instant::now();
  let mut s=S{fm:HashMap::new(),em:HashMap::new(),nodes:0}; let sg=gens(&v); let r=val2(&mut s,&sg);
  println!("{:?} S=T/2 quiet={} {} nodes={} {:.2}s",v.iter().map(|x|x*2).collect::<Vec<_>>(),quiet(&sg),["P","N","UNKNOWN"][r as usize],s.nodes,t0.elapsed().as_secs_f64()); }
