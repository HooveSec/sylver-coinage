# Results: independent checks around the opening 16

Every claim below has one status label. The labels are defined in
[README.md](README.md#status-legend). The numbers are copied from the
run records in `certificates/`; nothing is estimated. Dates are
America/New_York (ET).

**Nothing here resolves the opening 16, and nothing here is a new answer.**
Reply 38 was already answered by search in Rudi Cilibrasi's campaign
(72 answers it). Our reply-38 work checks his search witnesses
with independent code. It does not find new answers. Reply 40 has no
known answer, and we found none.

## 1. Reply 38: the seven deep witnesses of {16,38,72}

Cilibrasi's record (`sylver/campaigns/c72-2026-10-18/RESULT.md`, upstream
`main` at the 2026-10-29 merge of #74) certifies 61 of the 68 Quiet End
obligations of {16,38,72}. The other seven are long children of 72 whose
P answers rest on search witnesses that were never replayed. They need
75–135 GB of RAM to replay natively. Each row below is one of those seven
positions {16,38,72,c} together with its winning reply w. The claim is
that {16,38,72,c,w} is P, so w answers c.

Our route: fin3 (our solver) finds the position P and writes a P-journal Q.
Then two checkers that share no code with fin3 or with each other check
that Q is a closed P-set (see [METHODS.md](METHODS.md)). These positions
replay **known** search witnesses, so a certificate here is a check of a
known claim, not a new resolution.

| c + w | F | fin3 verdict (date) | |Q| (P-set) | checkq / checkq_lm (C) | checkq2 (Rust) | ender uses (distinct) | Ender-free re-check | Status |
|---|---:|---|---:|---|---|---:|---|---|
| 68 + 257 | 355 | P, 1,414,281,139 nodes (2026-10-19) | 32,157,514 | checkq OK 2026-10-19 07:34 | OK 2026-10-19 09:58 | 87 (77) | Q' = Q + 109,111 keys = 32,266,625: checkq OK 2026-10-19 11:27, checkq2 OK 2026-10-19 21:08, 0 ender uses | **TWO-CHECKER CERTIFIED, ender-free** |
| 100 + 245 | 367 | P, 3,619,911,438 nodes (2026-10-19) | 69,070,884 | checkq OK 2026-10-25 23:04 | OK 2026-10-28 09:42 | 119 (108) | Q' = Q + 266,327 keys = 69,337,211: checkq_lm OK 2026-11-02 07:12, checkq2 OK 2026-11-04 07:10, 0 ender uses | **TWO-CHECKER CERTIFIED, ender-free** |
| 84 + 139 | 245 | P, 1,840,890,550 nodes (by 2026-10-29) | 38,926,175 | checkq_lm OK 2026-11-09 15:08 | OK 2026-11-06 15:07 | 89 (81) | not done | **TWO-CHECKER CERTIFIED, conditional on the ender theorem** |
| 78 + 367 | 505 | P, 2,673,033,636 nodes (2026-10-21) | 53,887,031 | checkq_lm OK 2026-11-23 20:45 | OK 2026-11-14 20:21 | 113 (100) | not done | **TWO-CHECKER CERTIFIED, conditional on the ender theorem** |
| 119 + 154 | 257 | P, 2,366,972,175 nodes (2026-11-01 17:04) | 47,477,705 | not run yet | OK 2026-11-26 16:54 | 101 (88) | not done | **ONE CHECKER (checkq2) + fin3, conditional on the ender theorem**; the second checker is still to run |
| 116 + 229 | 367 | P, 2,180,361,734 nodes (reported 2026-10-27) | 79,864,861 | not run | not run | — | — | **SINGLE-METHOD SEARCH (fin3), unverified**, uses the ender rule |
| 106 + 137 | 269 | P, 2,507,963,493 nodes (2026-11-04 13:10) | 50,033,736 | not run | not run | — | — | **SINGLE-METHOD SEARCH (fin3), unverified**, uses the ender rule |

Children checked (every legal move of every key in Q, each refuted
through a reply into Q):

| Case | children (Q) | children (Q') | checkq/checkq_lm probes | checkq2 probes |
|---|---:|---:|---:|---:|
| 68+257 | 2,357,692,379 | 2,362,667,174 | 29,722,533,543 (Q'; 29,753,083,007) | 43,613,598,018 (Q'; 43,648,860,023) |
| 100+245 | 5,588,631,706 | 5,602,131,185 | 77,110,335,011 (Q'; 77,203,281,554) | 112,373,201,114 (Q'; 112,485,471,152) |
| 84+139 | 2,621,908,407 | — | 35,208,373,526 | 43,296,827,946 |
| 78+367 | 4,505,754,559 | — | 59,013,320,776 | 95,858,039,495 |
| 119+154 | 3,220,355,396 | — | — | 52,951,933,825 |

Agreement between the two checkers on every two-checker row:
- Same |Q|, `dup=0`, the same `children` count exactly, and the same
  number of ender uses.
- The same *set* of ender children, compared from the logged ender
  files in `certificates/reply38/enders/`.
- The probe counts differ because the two checkers try replies in
  different orders. That is expected.

For 100+245 and 78+367, checkq2's ender *log* has 3 and 4 more lines than
its final counter (122 vs 119 and 117 vs 113). Those runs resumed from
checkpoints after box restarts (`from=21000000` in the log), so the keys
after the last checkpoint were logged twice. The distinct sets match
checkq's exactly.

**Ender-free re-checks.** In the ender-free re-checks, each distinct
ender child Y was solved with fin3 with the ender rule off (ENDER=0):
- 68+257: 77 of 77 children are N.
- 100+245: 108 of 108 children are N.

Their P-journals were merged into Q to give Q', and both checkers then
re-checked Q' with the theorem disabled. Each reported 0 ender uses
(`certificates/reply38/ender_free/`).

**Journals and checkers.** The journals are 0.5–1.3 GB each, too big for
git. They are listed with their size and sha256 in
`certificates/LARGE_ARTIFACTS.tsv`. The checker binaries' sha256 (the
first 16 hex digits appear in every run log) are in
`certificates/BINARIES.tsv`:
- checkq `55ea26d1…`
- checkq_lm `317c9f70…`
- checkq2 `ca169909…`

**Comparison with the source.** Every verdict agrees with Cilibrasi's
search result for the same position. Literature check, 2026-11-26:
- Upstream `main` still lists all seven as "search witness only".
- No later upstream PR or commit: the last merge is #74 on 2026-10-29.
- Sicherman's news page has no item after 2026-10-18.
- arXiv has 0 hits for "sylver coinage".

As far as these public records show, these are the first independent
checked certificates of any of the seven. They still certify known
answers.

## 2. Reply 40

| Claim | Evidence | Status |
|---|---|---|
| 40 has no odd winning reply: {16,40} = 8·{2,5}. Every odd z ≥ 5 gives a quiet ender with Frobenius number > 1, so N. {16,40,3} = {3,16} is N by 2. | short argument (Quiet End Theorem); in draft PR #1 on the fork | **PROVED (short argument)**, already in draft PR #1 |
| Replies 10, 14, 18, 22, 26, 30, 34 to {16,40} are N (none of them answers 40) | `even3` (g=2 solver + QET + pattern2), one run each, 2026-11-01 to 2026-11-03. pfail = 0, 0, 0, 1, 2, 4, 9 failed pattern calls; failed calls are treated as UNKNOWN by design | **SINGLE-METHOD SEARCH (unverified)**. Upstream already **certifies** all seven as failing (r40-2026-10-23 table), so these rows add only an independent search agreement |
| Replies 38, 42, 46 to {16,40} | even3 TIMEOUT (wall 122,237 s, 92,233 s, 88,674 s; per-try limit TRY_TO=10,800 s; wall includes box suspensions) | **TIMEOUT/UNKNOWN** on our side. Upstream certifies all three as failing: 38 by 28, 42 by 23, 46 by 20 |
| Even replies 50–250 (≡ 2 mod 4) to {16,40} | 51 runs, all `ERR(rc=101)` within 2–10 s (the even3 child crashed at start; no value) | **NOT RUN (error)**: no claim |
| Is there any answer to 40? | — | **OPEN** |

## 3. Odd answers to the replies 42, 44 and 46 (`g4lane`, fin3)

These lanes test the odd candidates a for {16,r,a}. Each is a finite
position, solved by fin3 with the ender rule. The candidates are the
odd gaps above 1 of the half-pair, as in Cilibrasi's ledger: {8,21} for
42 (39 candidates), {4,11} for 44 (9) and {8,23} for 46 (43). Any
N result only rules out that one answer. None of these lanes found a P
child, so **no answer** to 42, 44 or 46 was found.

| Reply | N (a fails), single-method | TIMEOUT / UNKNOWN (never counted as N) | Not tried yet | Even candidates |
|---|---|---|---|---|
| 42 | 3, 7 (coprime-pair enders, from cache), 5, 9, 11, 13, 15, 17, 19, 23, 25, 27, 31, 33, 35, 39, 41, 43, 47, 49, 57 (21 of 39) | 51, 55, 59, 65, 67, 73, 75, 81, 83, 89, 91, 97, 99, 107, 115, 123, 131, 139 (18) | none | 64 even replies 4–246: all `ERR(rc=101)`, no claim |
| 44 | 3 (from cache), 5, 7, 9, 13, 17, 21, 25, 29 (all 9) | — | — | 60 even replies 10–250: all `ERR(rc=101)`, no claim |
| 46 | 3, 5, 7 (from cache), 9, 11, 13, 15, 17, 19, 21, 25, 27, 29, 33, 35, 37, 41 (17 of 43) | 43, 45, 49, 51, 53 (5) | 57 onwards (21); 57 running at packaging time | not started |

Status: **SINGLE-METHOD SEARCH (unverified)** for every N; **TIMEOUT/UNKNOWN**
for the timeouts; the answers to 42, 44 and 46 are **OPEN**.

The cache rows are N because a, 16 is a coprime pair (an ender) once the
reply is absorbed. For example, 3 and 7 divide 42, so {16,42,7} = {7,16}.

Comparison with Cilibrasi's search (`o16-2026-10-09/scan/`, fetched
2026-11-26; also by search, not certified). We found no conflicts:
- **Reply 42:** his ledger swept 22 of 39 candidates. His rescan then
  found 65, 59, 73, 55, 67, 75, 81, 83, 91, 89, 99, 107, 115 and 97 N
  and stopped with 123, 131 and 139 left.
  - 14 of our 18 timeouts are N in his rescan.
  - 51 must be among his 22 ledger rows, since 22 + 14 + 3 = 39. That
    is an inference from the counts; the ledger file does not list the
    swept candidates.
  - 123, 131 and 139 are open in both searches.
- **Reply 44:** he swept all 9 candidates, with no answer. That matches
  ours.
- **Reply 46:** he swept 18 candidates, then found 59, 41, 45, 49, 57,
  67, 53 and 75 N.

Where both searches say N, two independent engines agree, but both are
discovery searches. Neither one is a certificate.

## 4. Items already in draft PR #1 on the fork (approved by Nick; relayed 2026-11-05)

[HooveSec/sylver-coinage#1](https://github.com/HooveSec/sylver-coinage/pull/1)
(draft, open, created 2026-11-06) carries these three items, in
`sylver/campaigns/external-2026-10-19/`. They are listed here for
completeness. This package does not duplicate them.

| Item | Evidence | Status |
|---|---|---|
| Cilibrasi's answers 14→8, 22→12, 24→10 re-derived. {16,20,46} is P, so 46 also answers 20 | `vchk` (C brute force + QET) with five P-lemmas proved by exact pattern cycles ({8,10,12,14} period 8, {8,10,22} period 8 from 49, {8,12,18,22} 40, {8,12,26,30} 240, {8,12,34,38} 720; 0 odd winning moves each). The even side of each lemma is Blok's published pairing proof. Lemma-free vchk returns U (`certificates/theory/`) | **INDEPENDENT CONFIRMATION of known answers**, conditional on QET + the five lemmas. {16,20,34} (his certified node T) is U here, not a disagreement |
| Five of Cilibrasi's finite ledger witnesses replayed: {16,23,36}, {11,16,58}, {16,17,90}, {13,16,140} P with solve.rs and vchk (identical node counts); {16,21,156} P with vchk only (solve.rs hit its 2 GB cap) | `second_method.log` | **CONFIRMED (two engines; one for {16,21,156})** |
| Reply 40 has no odd answer | section 2 | **PROVED (short argument)** |
| Blok 14-even §5.2: in {14,22,24,32,34,40} the entry 251 should be 25. {..,25} is P (F=55, 18,010 nodes); {..,251} is N by 25 (F=281); 25 is the only odd winning move ≤ 401 (even moves not scanned) | solve.rs + vchk, `typo_check.txt` | **CONFIRMED (two engines)**. Literature correction, no effect on {16} |

## 5. Other runs (no claim)

- even3 {16,26,88}, an independent re-check of Cilibrasi's certified
  U = {16,26,88}: finished 2026-11-26 12:40 as **UNKNOWN**, with
  41,780,348 nodes and 19 failed pattern calls. It neither agrees nor
  disagrees.
- vchk {16,22}, a lemma-free brute force: still running at packaging
  time with no verdict. {16,22} N via 12 is already confirmed in
  section 4.
