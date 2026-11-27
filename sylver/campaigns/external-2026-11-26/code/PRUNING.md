# PRUNING.md — proven rules that decide Sylver Coinage positions cheaply
Spec for the solver worker. Written 2026-10-18 ~21:55 EDT by the Math Lab Sylver theory worker.
Notation: a position T is the set of named numbers; <T> the semigroup it generates; a *legal move* (gap) is x >= 1 with x not in <T>;
g(T) = gcd; for g(T)=1, t(T) = Frobenius number (largest gap). N = player to move wins, P = player to move loses. T+x = T with x named.
Every rule below says which source proves it and its EXACT hypotheses. Rules marked [LEMMA] import a published P-position and make
any result that uses them conditional; log which lemma fired.

> **Correction (important):** Hutchings's theorem says coprime {m,n} (other than {2,3}) is **N** (the player to move WINS), not P.
> The phrasing "two coprime numbers >= 5 lose for the player to move" / "{a,b} with a,b>=5 is P" seen in our chat and task notes is wrong.
> What is P is the single opening {p} for a prime p >= 5 (so naming p wins for the player who named it).

## R0. Terminal / trivial
- If t(T) = 1 (only legal move is 1, i.e. <T> = <2,3>) then T is **P**. Naming 1 loses, so never consider 1 as a candidate move. (rules of the game)
- Canonicalize: drop generators that are in the semigroup of the others; positions with equal <T> are the same position. Memo keys must be the semigroup (gap set / Kunz/Apéry coordinates), not the move list.

## R1. Enders are N  (Hutchings; Sicherman, "Theory and Practice of Sylver Coinage", Integers 2 (2002) #G02, §1; enders.html rev. 2020-12-26)
- Definition: g(T)=1 and every legal u < t(T) eliminates t (t in <T,u>).
- **Test (Weak Antisymmetry, proven iff in Integers 2002 §3 / enders.html):** T is an ender iff there is NO pair m < n with m+n = t such that both m and n are gaps.
  O(t) with a gap bitset: (gaps & reverse(gaps)) has no bit other than t/2.
- **Rule:** if g(T)=1, t(T) > 1 and T is an ender, then T is **N** (strategy stealing). Proof gives no winning move, so the rule is fine for VALUE search but a P-certificate that relies on it must cite it.
- Special case (Hutchings): {m,n} coprime, not {2,3}, is a (quiet) ender, hence N. Every {16, odd} is N by this rule, with zero search.
- Quiet ender = ender with t odd (Enders Theorem, Integers 2002 §3). Needed for R3.
- Measured (theory/ender_ab.tsv: ../solve vs theory/solve_e on 383 finite positions = 9 {16,odd}, the Blok-typo pair, 372 Blok 6-16 finite rows): 0 value disagreements. Root-level ender hits make every 2-generator child free (all {16,odd}: 0 nodes). Inside deeper searches the saving is only ~4% of nodes (node-weighted; 10-40% on tiny positions, ~1% on the 171k-node typo position). Worth adding as a leaf test (it is O(t) bit ops), not a big multiplier.

## R2. Prime-g rule  (Hutchings's p-theorem; Integers 2002 §1-2)
- {p} is P for prime p >= 5. Hence if a prime p >= 5 divides g(T) and p is legal, then T+p = {p} is P, so T is **N** (winning move p).
- Irrelevant inside {16,...} (g is a power of 2) but valid in general solvers.

## R3. Quiet End Theorem (QET) and the odd-move bound  (Winning Ways ch.18 [cited via Sicherman]; Integers 2002 §1; enders.html; semigroup proof: Amer. Math. Monthly 127 (2020), doi 10.1080/00029890.2020.1785254)
- **Statement:** if z is coprime with x and with y, then (S×x)+z is a quiet ender iff (S×y)+z is. z need NOT be legal in S. (Taking y = 1: (S×g)+z is a quiet ender iff S+z is.)
- **Consequence used by the solver (short positions):** let T = g·S with g(S)=1 and S a quiet ender. For any z coprime to g with z in <S> (z illegal in S, in particular every z > t(S)), S+z = S is a quiet ender, so T+z is a quiet ender, so T+z is N and z is **not** a winning move.
  => Candidate winning moves of T that are coprime to g are only the gaps z of S with z > 1 coprime to g. This holds for ANY g, not just g=2.
- {16,N}: N = 4j+2 => {16,N} = 2·{8,2j+1}; {8,odd} is coprime, so a quiet ender (Progression/Hutchings) => **every {16, 4j+2} is short**: its odd candidate answers are the odd gaps of {8,2j+1} above 1. For N=38: 35 odd + 63 even obligations (Cilibrasi r38 record). The genus of {8,k} is 7(k-1)/2, so the obligation count grows linearly in N.
  N = 8k (k odd) => {16,8k} = 8·{2,k}, quiet ender => the only odd candidates are odd z < k. Example {16,40} = 8·{2,5}: the only odd candidate is 3, and {16,40,3} = {3,16} is an ender => N. **So 40 has no odd answer; any answer to 40 must be even.**
  N = 4j, j odd => {16,4j} = 4·{4,j}, quiet ender => odd candidates are the odd gaps of {4,j} (all < 3j-4).
- Short g=2 position T: T is P iff every candidate (odd gaps of S above 1 and even moves 2u, u a gap of S) leads to N ("obligations"; Cilibrasi's term). The P-proof is a finite list of N-witnesses.
- Long positions (S not a quiet ender): no QET bound; for large z, T+z is not even an ender. Long g=2 positions are rarely P (known long P: {8,10,22}, {8,12,8n+2,8n+6}, {16,22,24,26,28,30,34}, {20,22,24,26,28,36}, {20,26,...,42}, a few of Blok's; ppos.html marks them *).

## R4. g = 2 Periodicity Theorem  (Winning Ways ch.18 "Periodicity Theorem"; Sicherman 1991 §5 "Pattern Theorem"; Integers 2002 §2)
- If g(T)=2, the values of T+x for odd x are eventually periodic, computable by a finite automaton; with τ = 2·t(T/2), analyzing an odd move x needs only odd replies in [x-τ, x+τ] (larger ones are eliminated by x, smaller ones eliminate x) plus the finitely many even replies (Integers 2002 §2 example: {8,10,22}, τ=14).
- Rule: a proven exact cycle (full window state equality, not hash equality) with no odd winning move in the pre-period + one period => no odd winning move at all. Our pattern.rs implements this (validated on Sicherman 1996 Table 2 periods).
- Not available for g >= 3 (Sicherman 1991 §5; Blok 14-even §11: "no analogue for g=3"). For g=4 (e.g. {16,4j}) there is no automaton; P-claims need every move class covered by R3/R5.

## R5. Known-P lookup, partner rule and descendants  [LEMMA when the P-position is imported]
- **One-move lookahead:** if some legal x has <T+x> equal to a known P-position Q, then T is **N** (move x). E.g. {12,16,20}+8 = {8,12} (Cilibrasi 2026-08-27, news.html).
- **Absorption nodes:** for an even position T (no 2) with 4, 6 legal: T+4+6 = {4,6} (P) since every even number >= 4 is in <4,6>; so T+4 and T+6 are N. Likewise T+8+14 = {8,14} when <T> ⊆ <8,14>, T+14 reaches {12,14,16} from T ⊇ {12,16} if <T> ⊆ <12,14,16>, etc. Implement as: canonicalize T+x+y and look it up in the P-table.
- **Partner rule (trivial but useful):** T+a+b = T+b+a. If that position is P, record BOTH T+a and T+b as N (when the second move is legal in each). Cilibrasi's ladder: with 16 in every position, a+16 is eliminated by a, so among the rungs a, a+16, a+32, ... of {16,r,·} at most one is P.
- **Descendant rule** (Bras-Amorós, Moskowitz, Ponomarenko, "Construction of an endgame rulebook for Sylver Coinage using trees of numerical semigroups", Comm. Algebra 2025, Cor. 4.1): if S is P and T ⊂ S with S = <T, g> (single generator g), then T is N via g, and so is every descendant T'' with S = <T'', g>. That is R5's lookup applied to whole subtrees; useful for P-only memos.
- **Store only P in RAM** (same paper §4; Cilibrasi kunz_solver --p-table): P-positions are ~6% of states; N can be re-proved from a stored P child. Biggest memory lever available; candidate for even2/g4 memory design.

## R6. Pairing strategies (P families)  [LEMMA]
- {8,12} is P and {8,12,8n+2,8n+6} is P for the SAME n >= 1 (Blok 2021 g=2 paper §2: pairing of odd moves (4n+1, 4n+3), "one is available whenever the other is"; Sicherman 1991 §6/8). Our LESSONS: an over-broad version (any a,b) produced a false win in {8,30,34}. Check n is identical.
- {4,6}: P (QET, R3). {6,9}: P (QET). {8,14}: P (QET + finite search; Blok g=2 table 2.7).

## R7. Published P-positions usable as lemmas  [LEMMA]
- Full list: sicherman.net/sylver/ppos.html (updated 2026-09-28; ~250 positions, long ones starred) → already partly in known.tsv / known_blok.tsv.
- Additional 2026 certified P (Cilibrasi repo github.com/rudi-cilibrasi/sylver-coinage, two engines, matching state counts): U={16,26,88}, Y={16,28,58}, Z={16,30,56}, Z'={16,30,40,44}, W={16,26,62,98}, I={16,20,22,24}, B24={16,24,38,44}, B28={16,28,38,40}, B56={16,38,56,60}, {16,26,56,62,66}, {16,26,82,88,701} (finite). By search only (NOT certified): {16,38,72}, {16,28,50}.
- Known errata: Blok 14-even §5.2 {14,22,24,32,34,40} lists 251; the winning move is 25 (recomputed, below). WW Table 5 {9,10,12} should be [11,13,14,17] (news.html 2020-12-23). Sicherman ppos {18,24,28,30,34,40,50} → {18,24,28,30,40,50} (2021-10-21).

## Not pruning rules (do not implement as such)
- Single Win Theorem / Separation Lemma (Integers 2002 §5): existence results only.
- "Probably P" (glossary): heuristic label, never a value.
- Progression Hypothesis page: stated as a hypothesis, but Integers 2002 §4 gives a proof sketch of the ender classification for arithmetic progressions {a0..as}, s <= a0-1: s=1 quiet ender; a0=3,s=2 unquiet ender; a0>3: quiet ender iff s | (a0-2), else not an ender. Usable as a fast shortness test for progressions.
- Blok's "nearly short" observation (g=2 paper §4.2): question, not theorem.
