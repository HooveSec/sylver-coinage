// g=2 "pattern" solver (re-derivation of Sicherman's Pattern Theorem idea).
// pattern3 (Sylver worker, 2026-10-19): pattern2 + graceful memory cap. memo/fm/scan are pure caches (a value never depends on
// what is cached), so instead of letting a HashMap doubling abort the process under `ulimit -v`, we evict before a grow
// would exceed the budget: memo keeps the entries with the largest o (nearest the frontier), fm/scan drop half.
// Budget: PMEM_MB, else 0.8*(soft RLIMIT_AS - stack) - 200 MB, else 2048 MB. Evictions are reported on stderr ([evict]);
// stdout is unchanged from pattern2. PMEM_MB tiny = stress test of the eviction path (results must not change).
// Position <2S, odd part>, where the odd elements >= o are exactly o + 2A, with A an S-ideal (0 in A, A+S ⊆ A).
// For o >= 2t(S)+3 the game depends only on (S, A, o):
//   even move 2m (m gap of S)            -> (S+m, A closed under S+m, o)
//   odd move o+2a (a not in A, a<=t)     -> (S, A ∪ (a+S), o)
//   odd move o-2k, 1<=k<=t               -> (S, S ∪ (k+A), o-2k)
//   odd move o-2k, k>t                   -> (S, S, o-2k)   [aggregated by firstP(S)]
// Below the threshold, positions are solved exactly by the finite gap-bitset solver.
use std::collections::HashMap; use std::env;
type G=Vec<u64>;
fn get(g:&G,i:usize)->bool{ i<g.len()*64 && g[i>>6]>>(i&63)&1==1 }
fn frob(g:&G)->usize{ for w in (0..g.len()).rev(){ if g[w]!=0 {return w*64+63-g[w].leading_zeros() as usize;} } 0 }
fn from_in(inn:&[bool])->G{ let mut f=0; for i in 1..inn.len(){ if !inn[i]{f=i;} } let mut r=vec![0u64;f/64+1]; for i in 1..=f { if !inn[i]{ r[i>>6]|=1<<(i&63);} } r }
fn addg(g:&G,a:usize)->G{ let f=frob(g); let mut inn=vec![true;f+1]; for i in 0..=f { if get(g,i){inn[i]=false;} }
  if a<=f {inn[a]=true;} for n in a..=f { if inn[n-a]{inn[n]=true;} } from_in(&inn) }
fn gens(v:&[usize])->G{ let m=*v.iter().min().unwrap(); let lim=v.iter().max().unwrap()*m+5; let mut inn=vec![false;lim]; inn[0]=true;
  for &a in v { for n in a..lim { if inn[n-a]{inn[n]=true;} } } from_in(&inn) }
// ---- compact S / A as u128 masks over [0,t]
#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)] struct Sg{inn:u128,t:u8} // inn bit i: i in S (i<=t)
fn s_from_list(v:&[usize])->Sg{ let g=gens(v); let t=frob(&g); assert!(t<127); let mut inn=0u128; for i in 0..=t { if !get(&g,i){inn|=1<<i;} } Sg{inn,t:t as u8} }
fn s_has(s:Sg,i:usize)->bool{ i>s.t as usize || s.inn>>i&1==1 }
fn s_add(s:Sg,m:usize)->Sg{ let t=s.t as usize; let mut inn:Vec<bool>=(0..=t).map(|i|s_has(s,i)).collect(); if m<=t {inn[m]=true;}
  for n in m..=t { if inn[n-m]{inn[n]=true;} } let mut nt=0; for i in 0..=t { if !inn[i]{nt=i;} }
  let mut r=0u128; for i in 0..=nt { if inn[i]{r|=1<<i;} } Sg{inn:r,t:nt as u8} }
fn a_has(a:u128,t:usize,i:usize)->bool{ i>t || a>>i&1==1 }
fn a_close(s:Sg,mut a:u128)->u128{ let t=s.t as usize; a&= if t>=127 {!0} else {(1u128<<(t+1))-1}; a|=1;
  for i in 0..=t { if a>>i&1==1 { for j in 1..=t-i { if s_has(s,j){ a|=1<<(i+j);} } } } a }
fn s_gens(s:Sg)->Vec<usize>{ let t=s.t as usize; let mut out=vec![]; for x in 1..=2*t+3 { if !s_has(s,x){continue;}
  if (1..x).any(|y| s_has(s,y)&&s_has(s,x-y)) {continue;} out.push(x);} out }
struct St{ lv:HashMap<u32,u64>, memo:HashMap<(Sg,u128,u32),bool>, fm:HashMap<G,bool>, scan:HashMap<Sg,(u32,Option<u32>)>,
  budget:usize, fm_heap:usize, ev_memo:u64, ev_fm:u64, ev_scan:u64 }
fn buckets(cap:usize)->usize{ if cap==0 {0} else if cap<4 {4} else if cap<8 {8} else { (cap*8/7).next_power_of_two() } }
const SZ_MEMO:usize=std::mem::size_of::<((Sg,u128,u32),bool)>()+1; const SZ_FM:usize=std::mem::size_of::<(G,bool)>()+1;
const SZ_SCAN:usize=std::mem::size_of::<(Sg,(u32,Option<u32>))>()+1;
fn est(st:&St)->usize{ buckets(st.memo.capacity())*SZ_MEMO+buckets(st.fm.capacity())*SZ_FM+st.fm_heap+buckets(st.scan.capacity())*SZ_SCAN }
fn nextb(cap:usize,sz:usize)->usize{ let b=buckets(cap); (if b==0 {4} else {2*b})*sz }
fn evict_memo(st:&mut St){ let n=st.memo.len(); if n<16 { st.memo.clear(); st.ev_memo+=1; return; }
  // keep the ~40% of entries with the largest o; ties at the threshold level are kept up to the quota (no full clear -> no thrash)
  let keep=n*4/10; let mut os:Vec<u32>=st.memo.keys().map(|k|k.2).collect(); let i=n-1-keep; os.select_nth_unstable(i); let thr=os[i];
  let above=os.iter().filter(|&&o|o>thr).count(); let mut quota=keep.saturating_sub(above);
  st.memo.retain(|k,_| if k.2>thr {true} else if k.2==thr && quota>0 {quota-=1; true} else {false});
  st.ev_memo+=1; eprintln!("[evict] memo {} -> {} (o>{}) est={}MB budget={}MB",n,st.memo.len(),thr,est(st)>>20,st.budget>>20); }
fn evict_fm(st:&mut St){ let n=st.fm.len(); let mut i=0usize; let mut freed=0usize;
  st.fm.retain(|k,_|{ i+=1; if i%2==0 {true} else { freed+=k.capacity()*8+16; false } }); st.fm_heap=st.fm_heap.saturating_sub(freed);
  st.ev_fm+=1; eprintln!("[evict] fm {} -> {} est={}MB budget={}MB",n,st.fm.len(),est(st)>>20,st.budget>>20); }
fn evict_scan(st:&mut St){ let n=st.scan.len(); let mut i=0usize; st.scan.retain(|_,_|{ i+=1; i%2==0 });
  st.ev_scan+=1; eprintln!("[evict] scan {} -> {}",n,st.scan.len()); }
// make room before an insert: if the table is full (next insert would grow it) and the grow would break the budget, evict.
fn room(st:&mut St,which:u8){ for _ in 0..4 {
  let (len,cap,sz)=match which{0=>(st.memo.len(),st.memo.capacity(),SZ_MEMO),1=>(st.fm.len(),st.fm.capacity(),SZ_FM),_=>(st.scan.len(),st.scan.capacity(),SZ_SCAN)};
  if len<cap { return; } if est(st)+nextb(cap,sz)<=st.budget { return; }
  // over budget: shrink the biggest consumer first, then this table
  let mb=buckets(st.memo.capacity())*SZ_MEMO; let fb=buckets(st.fm.capacity())*SZ_FM+st.fm_heap;
  if mb>=fb && st.memo.len()>0 { evict_memo(st); } else if st.fm.len()>0 { evict_fm(st); } else if st.memo.len()>0 { evict_memo(st); } else { evict_scan(st); } } }
fn budget()->usize{ if let Some(m)=std::env::var("PMEM_MB").ok().and_then(|x|x.parse::<usize>().ok()) { return m<<20; }
  let stack=std::env::var("PSTACK_MB").ok().and_then(|x|x.parse::<usize>().ok()).unwrap_or(1024)<<20;
  if let Ok(l)=std::fs::read_to_string("/proc/self/limits") { for line in l.lines(){ if line.starts_with("Max address space") {
    let f:Vec<&str>=line.split_whitespace().collect(); if let Ok(v)=f[3].parse::<usize>() { return (((v.saturating_sub(stack)) as f64*0.8) as usize).saturating_sub((200usize<<20).min(v/4)).max(64<<20); } } } }
  2048<<20 }
fn fwin(st:&mut St,g:&G)->bool{ let f=frob(g); if f<=1 {return false;} if let Some(&v)=st.fm.get(g){return v;}
  let mut r=false; let mut x=f; while x>=2 { if get(g,x){ if !fwin(st,&addg(g,x)){r=true;break;} } x-=1; } room(st,1); st.fm_heap+=g.len()*8+16; st.fm.insert(g.clone(),r); r }
fn exact(st:&mut St,s:Sg,a:u128,o:usize)->bool{ let t=s.t as usize; let mut v:Vec<usize>=s_gens(s).iter().map(|x|2*x).collect();
  if v.is_empty(){ v.push(2); }
  for i in 0..=t+1+2*t+2 { if a_has(a,t,i) { v.push(o+2*i);} } fwin(st,&gens(&v)) }
fn first_p(st:&mut St,s:Sg,lim:usize)->bool{ // exists odd x in [3,lim] with V(S,S,x)=P
  if lim<3 {return false;}
  loop { let (up,fp)=*st.scan.get(&s).unwrap_or(&(1,None)); if let Some(p)=fp { if p as usize<=lim {return true;} }
    if up as usize>=lim { return false; } let x=up as usize+2; let v=val(st,s,s.inn,x);
    if !st.scan.contains_key(&s) { room(st,2); } let e=st.scan.entry(s).or_insert((1,None)); e.0=x as u32; if !v && e.1.is_none(){ e.1=Some(x as u32);} }
}
fn val(st:&mut St,s:Sg,a:u128,o:usize)->bool{ let t=s.t as usize; let a=a_close(s,a);
  if o<2*t+3 { return exact(st,s,a,o); }
  if let Some(&v)=st.memo.get(&(s,a,o as u32)){return v;}
  let mut r=false;
  // even moves
  for m in 1..=t { if r {break;} if s_has(s,m){continue;} let s2=s_add(s,m); if !val(st,s2,a,o){r=true;} }
  // window odd moves
  for i in 1..=t { if r {break;} if a_has(a,t,i){continue;} if !val(st,s,a|(1u128<<i),o){r=true;} }
  // smaller odd, k<=t
  for k in 1..=t { if r {break;} if o<2*k+3 {break;} let mut b=s.inn; for j in k..=t { if a_has(a,t,j-k){b|=1<<j;} } if !val(st,s,b,o-2*k){r=true;} }
  if !r && o>=2*t+2+3 { r=first_p(st,s,o-2*t-2); }
  room(st,0); st.memo.insert((s,a,o as u32),r);
  { let mut h=(s.inn as u64)^((s.inn>>64) as u64).rotate_left(17)^(a as u64).rotate_left(29)^((a>>64) as u64).rotate_left(41)^(r as u64); h=h.wrapping_mul(0x9E3779B97F4A7C15); h^=h>>29;
    let e=st.lv.entry(o as u32).or_insert(0); *e=e.wrapping_add(h); } r }
// Reachable abstract states (S,A) from (S0,S0).
fn reach(s0:Sg)->Vec<(Sg,u128)>{ let mut seen:std::collections::HashSet<(Sg,u128)>=Default::default(); let mut q=vec![(s0,a_close(s0,s0.inn))];
  while let Some((s,a))=q.pop(){ if !seen.insert((s,a)){continue;} let t=s.t as usize;
    for m in 1..=t { if !s_has(s,m){ let s2=s_add(s,m); q.push((s2,a_close(s2,a))); } }
    for i in 1..=t { if !a_has(a,t,i){ q.push((s,a_close(s,a|(1u128<<i)))); } }
    for k in 1..=t { let mut b=s.inn; for j in k..=t { if a_has(a,t,j-k){b|=1<<j;} } q.push((s,a_close(s,b))); }
    q.push((s,a_close(s,s.inn))); }
  let mut v:Vec<_>=seen.into_iter().collect(); v.sort_by_key(|(s,a)|(s.t,s.inn,*a)); v }
fn main(){ let args:Vec<String>=env::args().collect();
  let mode=args[1].clone(); let v:Vec<usize>=args[3..].iter().map(|x|x.parse::<usize>().unwrap()/2).collect(); let lim:usize=args[2].parse().unwrap();
  let child=std::thread::Builder::new().stack_size(std::env::var("PSTACK_MB").ok().and_then(|x|x.parse::<usize>().ok()).unwrap_or(1024)<<20).spawn(move||{
  let s=s_from_list(&v); let mut st=St{lv:HashMap::new(),memo:HashMap::new(),fm:HashMap::new(),scan:HashMap::new(),budget:budget(),fm_heap:0,ev_memo:0,ev_fm:0,ev_scan:0};
  eprintln!("pattern3 budget={}MB",st.budget>>20);
  eprintln!("S={:?} t={} gens={:?}",v,s.t,s_gens(s));
  if mode=="cycle" { // rigorous: full state vector over all reachable (S,A), exact window equality
    let r=reach(s); let ss:Vec<Sg>={let mut x:Vec<Sg>=r.iter().map(|p|p.0).collect(); x.sort_by_key(|q|(q.t,q.inn)); x.dedup(); x};
    let tmax=ss.iter().map(|q|q.t as usize).max().unwrap(); let thr=2*tmax+3; let win=tmax+2; // levels (odd steps) in window
    eprintln!("reachable states={} distinct S={} tmax={}",r.len(),ss.len(),tmax);
    let mut lv:Vec<Vec<u64>>=vec![]; let mut fl:Vec<Vec<u64>>=vec![]; let mut idx:HashMap<u64,Vec<usize>>=HashMap::new();
    let mut x=3; let mut li=0usize;
    while x<=lim { let mut bv=vec![0u64;(r.len()+63)/64]; for (i,&(s2,a2)) in r.iter().enumerate(){ if val(&mut st,s2,a2,x){bv[i>>6]|=1<<(i&63);} }
      if !val(&mut st,s,a_close(s,s.inn),x) { println!("odd winning move {}",x); }
      let mut fv=vec![0u64;(ss.len()+63)/64]; for (i,&q) in ss.iter().enumerate(){ let l=x as i64-2*q.t as i64-2; if l>=3 && first_p(&mut st,q,l as usize){fv[i>>6]|=1<<(i&63);} }
      lv.push(bv); fl.push(fv);
      if x>=thr+2*win { let mut h:u64=0xcbf29ce484222325; for j in li+1-win..=li { for w in &lv[j] { h=(h^w).wrapping_mul(0x100000001b3);} } for w in &fl[li] { h=(h^w).wrapping_mul(0x100000001b3);}
        let e=idx.entry(h).or_default(); let mut found=None;
        for &pj in e.iter() { if fl[pj]==fl[li] && (0..win).all(|d| lv[pj-d]==lv[li-d]) { found=Some(pj); break; } }
        if let Some(pj)=found { let p0=3+2*pj; println!("EXACT CYCLE: state at n={} equals state at n={} => period {} (window {} levels, {} states, {} S)",x,p0,x-p0,win,r.len(),ss.len()); return; }
        e.push(li); }
      if x%2001==1 { let keep=x.saturating_sub(4*tmax+8) as u32; st.memo.retain(|k,_|k.2>=keep); st.lv.retain(|k,_|*k>=keep); }
      if x%20001==1 { eprintln!("n={} memo={}",x,st.memo.len()); }
      x+=2; li+=1; } println!("no cycle up to {}",lim); return; }
  if mode=="check" { // compare pattern recursion with exact solver
    let mut bad=0; let mut x=3; while x<=lim { let p=val(&mut st,s,s.inn,x); let e=exact(&mut st,s,s.inn,x); if p!=e {bad+=1; println!("MISMATCH n={} pattern={} exact={}",x,p,e);} x+=2; }
    println!("check done up to {} mismatches={}",lim,bad); }
  else if mode=="scan" { // prune memo, detect cycles in the level-hash window (heuristic cycle detector)
    let w=2*s.t as usize+4; let mut seen:HashMap<u64,usize>=HashMap::new(); let mut x=3; let mut wins=0;
    while x<=lim { if !val(&mut st,s,s.inn,x){ wins+=1; println!("odd winning move {}",x); }
      if x>w+3 { let mut h:u64=0xcbf29ce484222325; for o in (x-w..=x).step_by(2) { h=(h^st.lv.get(&(o as u32)).copied().unwrap_or(0)).wrapping_mul(0x100000001b3); }
        let mut fl:Vec<u64>=st.scan.iter().filter(|(_,v)|v.1.is_some()).map(|(k,_)|k.inn as u64 ^ k.t as u64).collect(); fl.sort(); for f in fl { h=(h^f).wrapping_mul(0x100000001b3); }
        if let Some(&p)=seen.get(&h) { println!("CYCLE? state at n={} equals n={} (period {})",x,p,x-p); }
        else { seen.insert(h,x); } }
      if x%2001==1 { let keep=x.saturating_sub(4*w) as u32; st.memo.retain(|k,_|k.2>=keep); st.lv.retain(|k,_|*k>=keep); }
      if x%20001==1 { eprintln!("n={} memo={} wins={}",x,st.memo.len(),wins); }
      x+=2; } println!("scan done up to {} wins={}",lim,wins); }
  else { let mut x=3; while x<=lim { if !val(&mut st,s,s.inn,x){ println!("odd winning move {}",x); if mode=="first" {return;} }
      if x%2001==1 { let keep=x.saturating_sub(16*s.t as usize+16) as u32; st.memo.retain(|k,_|k.2>=keep); st.lv.retain(|k,_|*k>=keep); }
      if x%10001==0 { eprintln!("n={} memo={}",x,st.memo.len()); } x+=2; } println!("no (more) odd winning moves up to {}",lim); }
  }).unwrap(); child.join().unwrap(); }
