/* Independent Sylver coinage checker (verification track). Plain C, explicit-bound bitsets.
   Usage: vchk [-q] a b c ...   (all even, gcd 2) -> evaluates T=2S via Quiet End Theorem
          vchk -f a b c ...     finite position brute force (gcd 1 required)
   Finite game: position = set of gaps (bitset 0..F). win = exists gap x>=2 with !win(after x).
   Closure after adding x: in |= in<<x, in<<2x, in<<4x,... (doubling), bounded by F. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#define MAXW 32
static size_t MBG=900;
typedef struct { uint64_t w[MAXW]; } BS;
static int W; /* active words */
static size_t HCAP; static uint64_t *HK; static uint8_t *HV; static size_t hused; static long long nodes, clears;
static int bget(const BS*b,int i){return (b->w[i>>6]>>(i&63))&1;}
static void bset(BS*b,int i){b->w[i>>6]|=1ULL<<(i&63);}
static void shl_or(BS*d,const BS*s,int k,int nbits){ /* d |= s<<k, truncated at nbits */
  int ws=k>>6, bs=k&63; for(int i=W-1;i>=ws;i--){ uint64_t v=s->w[i-ws]<<bs; if(bs&&i-ws-1>=0) v|=s->w[i-ws-1]>>(64-bs); d->w[i]|=v; }
  int lw=nbits>>6, lb=nbits&63; if(lw<W){ d->w[lw]&=(lb?((1ULL<<lb)-1):0); for(int i=lw+1;i<W;i++) d->w[i]=0; } }
/* in = representable set restricted to [0,nbits) */
static void close_add(BS*in,int x,int nbits){ if(x<nbits) bset(in,x); for(int k=x;k<nbits;k*=2){ BS t=*in; shl_or(in,&t,k,nbits); if(k>nbits) break;} }
static uint64_t hsh(const BS*b){uint64_t h=1469598103934665603ULL; for(int i=0;i<W;i++){h^=b->w[i]; h*=0x100000001b3ULL; h^=h>>29;} return h;}
static int hget(const BS*b){ uint64_t h=hsh(b)%HCAP; for(;;){ if(!HV[h]) return -1; if(!memcmp(&HK[h*W],b->w,8*W)) return HV[h]-1; h=(h+1)%HCAP; } }
static void hput(const BS*b,int v){ if(hused*10>HCAP*7){ memset(HV,0,HCAP); hused=0; clears++; }
  uint64_t h=hsh(b)%HCAP; while(HV[h]) h=(h+1)%HCAP; memcpy(&HK[h*W],b->w,8*W); HV[h]=v+1; hused++; }
static int NB; /* single global bit bound for whole run (must exceed every Frobenius number) */
static void hinit(size_t mb){ HCAP=(mb<<20)/(8*W+1); HK=malloc(HCAP*8*W); HV=calloc(HCAP,1); if(!HK||!HV){fprintf(stderr,"oom\n");exit(4);} }
/* in: representable set over [0,NB). returns 1 if mover wins */
static int fwin(const BS*in){ int r=hget(in); if(r>=0) return r; nodes++; r=0;
  for(int x=NB-1;x>=2;x--){ if(bget(in,x)) continue; BS c=*in; close_add(&c,x,NB); if(!fwin(&c)){r=1;break;} }
  hput(in,r); return r; }
static int frob_list(int*a,int n,int lim,char*rep){ memset(rep,0,lim); rep[0]=1; for(int i=0;i<n;i++) for(int k=a[i];k<lim;k++) if(rep[k-a[i]]) rep[k]=1;
  int f=-1; for(int k=0;k<lim;k++) if(!rep[k]) f=k; return f; }
static int finite_win(int*a,int n,int*best){ /* gcd(a)=1 */ int lim=1; for(int i=0;i<n;i++) if(a[i]>lim) lim=a[i]; lim=lim*lim+10; char*rep=malloc(lim);
  int f=frob_list(a,n,lim,rep); if(f<0){free(rep);return 0;} NB=f+1; W=NB/64+1; if(W>MAXW){fprintf(stderr,"F too big %d\n",f); exit(3);} hinit(MBG);
  BS in; memset(&in,0,sizeof in); for(int k=0;k<NB;k++) if(rep[k]) bset(&in,k); free(rep);
  *best=0; for(int x=NB-1;x>=2;x--){ if(bget(&in,x)) continue; BS c=in; close_add(&c,x,NB); if(!fwin(&c)){*best=x;return 1;} } return 0; }
/* ---- g=2 layer. S given as generator list; T=2S. ---- */
#define SMAX 4096
typedef struct { char r[SMAX]; int t; } SG; /* r[k]=1 if k in S (k<=t+1) */
static void sg_make(SG*s,int*g,int n){ int lim=SMAX; frob_list(g,n,lim,s->r); s->t=-1; for(int k=0;k<lim;k++) if(!s->r[k]) s->t=k; }
static int quiet(SG*s){ if(s->t<1||s->t%2==0) return 0; for(int m=0;m<=s->t;m++) if(s->r[m]==s->r[s->t-m]) return 0; return 1; }
/* finite game <2S,n>: elements {2s} U {n}. F <= n+2t */
static int odd_child_win(SG*s,int n){ if(n+2*s->t+2>=NB){fprintf(stderr,"odd child exceeds NB\n");exit(3);} int L=NB;
  BS in; memset(&in,0,sizeof in); for(int k=0;2*k<L;k++) if(k>s->t||s->r[k]) bset(&in,2*k); close_add(&in,n,NB); return fwin(&in); }
/* memo for g2 layer: small linear list */
typedef struct { char r[256]; int t; int v; int mv; } M2; static M2*m2; static int nm2;
static int verbose;
static int nlem; static char lem[20000][64]; static int lemused[20000];
static void loadlem(const char*fn){ FILE*f=fopen(fn,"r"); if(!f) return; char line[512]; while(fgets(line,512,f)){ char*p=strtok(line," \t\n"); if(!p) continue; char*v=strtok(NULL," \t\n"); if(!v||strcmp(v,"P")) continue; /* keep only all-even */
    int ok=1; for(char*q=p;*q;){ int x=strtol(q,&q,10); if(x%2) ok=0; if(*q==',') q++; else break; } if(ok&&nlem<20000){ strncpy(lem[nlem],p,63); nlem++; } } fclose(f); }
static int lemma(SG*s){ if(!nlem) return -1; char buf[512]; int L=0; int t=s->t; int m0=0;
  for(int x=1;x<=t+1+ (m0?m0:t+1);x++){ if(!(x>t||s->r[x])) continue; if(!m0) m0=x; int red=0; for(int y=1;y<x;y++){ int a=(y>t||s->r[y]), b=(x-y>t||s->r[x-y]); if(a&&b){red=1;break;} } if(!red) L+=sprintf(buf+L,L?",%d":"%d",2*x); if(x>t+m0) break; }
  for(int i=0;i<nlem;i++) if(!strcmp(lem[i],buf)){ lemused[i]++; return i; } return -1; }
static int val2(SG*s,int depth,int*mv){ /* 0 P,1 N,2 U */
  if(s->t<0){ *mv=3; return 1; } /* S = all naturals: T={2}, reply 3 gives {2,3}=P */
  if(s->t>=256){fprintf(stderr,"t too big\n");exit(3);}
  for(int i=0;i<nm2;i++) if(m2[i].t==s->t && !memcmp(m2[i].r,s->r,s->t+1)){*mv=m2[i].mv; return m2[i].v;}
  if(depth>0 && lemma(s)>=0){ *mv=-1; m2=realloc(m2,sizeof(M2)*(nm2+1)); memcpy(m2[nm2].r,s->r,s->t+1); m2[nm2].t=s->t; m2[nm2].v=0; m2[nm2].mv=-1; nm2++; return 0; }
  int q=quiet(s), unk=!q, res=0; *mv=0;
  for(int m=s->t;m>=2;m--){ if(s->r[m]) continue;
    if(m%2){ if(!odd_child_win(s,m)){res=1;*mv=m;break;} }
    SG c;
    /* rebuild from S plus m */ memset(&c,0,sizeof c); { char rep[SMAX]; memset(rep,0,SMAX); for(int k=0;k<SMAX;k++) rep[k]=(k>s->t)||s->r[k];
      for(int k=m;k<SMAX;k++) if(rep[k-m]) rep[k]=1; memcpy(c.r,rep,SMAX); c.t=-1; for(int k=0;k<SMAX;k++) if(!rep[k]) c.t=k; }
    int cm; int cv=val2(&c,depth+1,&cm); if(cv==0){res=1;*mv=2*m;break;} if(cv==2) unk=1; }
  /* note: even move 2 (m=1) gives S=N -> T={2} which is N, never winning; skipped */
  if(!res && !q){ /* long position: try odd n in S up to bound as finite games */
    int B=getenv("VODDB")?atoi(getenv("VODDB")):4*s->t+40; for(int n=3;n<=B;n+=2){ if(n<=s->t && !s->r[n]) continue; if(n+2*s->t+2>=NB) break; if(!odd_child_win(s,n)){res=1;*mv=n;break;} } }
  if(!res && unk) res=2;
  if(verbose) { fprintf(stderr,"%*sS(t=%d) -> %c mv=%d\n",depth,"",s->t,"PNU"[res],*mv); }
  m2=realloc(m2,sizeof(M2)*(nm2+1)); memcpy(m2[nm2].r,s->r,s->t+1); m2[nm2].t=s->t; m2[nm2].v=res; m2[nm2].mv=*mv; nm2++;
  return res; }
int main(int argc,char**argv){ int ai=1, fin=0; size_t mb=900;
  if(getenv("VMB")) mb=atoi(getenv("VMB")); if(getenv("VV")) verbose=1;
  if(argc>1&&!strcmp(argv[1],"-f")){fin=1;ai++;}
  int a[64],n=0; for(;ai<argc;ai++) a[n++]=atoi(argv[ai]);
  MBG=mb;
  /* note: key stride fixed at W words of current game; W may change across odd children, so clear table when W changes */
  if(getenv("VLEM")) loadlem(getenv("VLEM"));
  if(fin){ int b; int r=finite_win(a,n,&b); printf("FINITE {"); for(int i=0;i<n;i++) printf("%s%d",i?",":"",a[i]); printf("} %s move=%d nodes=%lld clears=%lld\n",r?"N":"P",b,nodes,clears); return 0; }
  int g[64]; for(int i=0;i<n;i++){ if(a[i]%2){fprintf(stderr,"need even\n");return 2;} g[i]=a[i]/2; }
  SG s; sg_make(&s,g,n); { int B=getenv("VODDB")?atoi(getenv("VODDB")):4*s.t+40; NB=B+2*s.t+3; if(NB<3*s.t+4) NB=3*s.t+4; W=NB/64+1; if(W>MAXW){fprintf(stderr,"NB too big\n");return 3;} hinit(MBG);}  int mv; int r=val2(&s,0,&mv);
  printf("G2 {"); for(int i=0;i<n;i++) printf("%s%d",i?",":"",a[i]); printf("} quiet=%d %c move=%d nodes=%lld clears=%lld lemmas_used=",quiet(&s),"PNU"[r],mv,nodes,clears); int any=0; for(int i=0;i<nlem;i++) if(lemused[i]){printf("%s{%s}",any?";":"",lem[i]);any=1;} printf(any?"\n":"none\n"); return 0; }
