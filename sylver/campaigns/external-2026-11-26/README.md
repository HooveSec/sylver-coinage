# Independent certificates and checks around the opening 16

This folder is that contribution package, at
`sylver/campaigns/external-2026-11-26/` in
[HooveSec/sylver-coinage](https://github.com/HooveSec/sylver-coinage), a
fork of [rudi-cilibrasi/sylver-coinage](https://github.com/rudi-cilibrasi/sylver-coinage).
It sits next to `sylver/campaigns/external-2026-10-19/`, the three items
from PR #1, now on this fork's main. Those three items are not copied
here. [RESULTS.md §4](RESULTS.md#4-items-already-in-draft-pr-1-on-the-fork-approved-by-nick-relayed-2026-11-05)
cross-references them. Status labels in RESULTS.md are the package's
labels, unchanged.

**No open problem is claimed solved.** Opening 16 remains open. 40 is
still the lowest reply to 16 with no known answer.

## What this adds over the upstream repository

1. **Checked certificates for four of the seven reply-38 search witnesses.**
   Upstream certifies 61 of the 68 obligations of {16,38,72}. The
   remaining seven rest on search witnesses that need 75–135 GB to
   replay.
   - We re-solved all seven with our own solver (fin3).
   - Two independent checkers verified the P-sets of four of them in
     bounded memory (under 1.5 GB for most runs): 68+257, 100+245,
     84+139 and 78+367.
   - For two of them, 68+257 and 100+245, the check is also free of the
     ender theorem.
   - A fifth, 119+154, has one checker so far.

   These certify **known** answers. They are not new resolutions. See
   [RESULTS.md §1](RESULTS.md#1-reply-38-the-seven-deep-witnesses-of-163872).
2. **A certificate format and checker for P-table searches.** It checks
   "every key of Q is P" in memory bounded by |Q|. That fills the gap
   noted upstream, where `--verify-memo` cannot run with `--p-table`.
   See [METHODS.md](METHODS.md#p-set-certificate-and-the-checkers).
3. **Search-only negative results** for odd answers to the replies 42,
   44 and 46, and for replies 10–34 to {16,40}. They are labelled
   unverified. Upstream already certifies the {16,40} rows; the others
   agree with upstream's own search where they overlap
   ([RESULTS.md §2–3](RESULTS.md#2-reply-40)).
4. Cross-references to the three items already in
   `sylver/campaigns/external-2026-10-19/` (PR #1). They are not
   duplicated here. The run logs that RESULTS.md §4 cites stay under
   `certificates/theory/`.

## Layout

| Path | Contents |
|---|---|
| `RESULTS.md` | every claim with its status label |
| `METHODS.md` | solvers, checkers, the ender-theorem handling, and validation |
| `certificates/reply38/` | per-case verdict lines (`*_cert_*.txt`) and run logs (`*.log`) from both checkers; fin3 verdicts and logs; ender logs (`enders/`, 16-byte records: 15-byte Kunz key + pad); the ender-free re-check (`ender_free/`) |
| `certificates/lanes/` | `g4lane_tries.tsv`: every attempt of the reply-40/42/44/46 sweeps, including timeouts and errors |
| `certificates/theory/` | vchk lemma runs, cycle proofs, the Blok 251→25 check, ledger replays |
| `certificates/tests/` | checker test-suite outputs (original and 2026-11-26 rebuild) |
| `certificates/LARGE_ARTIFACTS.tsv` | size + sha256 of the P-journals (Q, Q'), 0.5–1.3 GB each, **not in git** |
| `certificates/BINARIES.tsv` | sha256 of the checker and solver binaries used for the runs |
| `certificates/MANIFEST.sha256` | sha256 of every file in this package |
| `code/checkers/` | `checkq.c`, `checkq_lm.c`, `checkq2.rs` |
| `code/solvers/` | `fin3.rs`, `even3.rs`, `even2.rs`, `pattern2.rs`, `pattern3.rs`, `vchk.c`, `solve.rs`, plus earlier prototypes |
| `code/scripts/` | run wrappers, the g4lane driver, the Q / Q' builders |
| `code/tests/` | checker test suites (`tests.sh`, `tests_lm.sh`, `tests2.sh`, `mkneg.py`) |
| `code/validation/`, `code/theory/` | solver validation scripts and tables |

Scripts keep the absolute paths of the machine they ran on
(`/workspace/sylver/...`). They are the scripts exactly as run. Edit the
paths to rerun them.

## Reproducing

Build (gcc ≥ 12, rustc ≥ 1.80; no crates):
```
gcc -O2 -o checkq code/checkers/checkq.c
gcc -O2 -o checkq_lm code/checkers/checkq_lm.c
rustc -O -o checkq2 code/checkers/checkq2.rs
rustc --edition 2021 -O -o fin3 code/solvers/fin3.rs
```
`checkq2.rs` builds with the default edition. `fin3.rs` calls `try_into`,
which the 2015 prelude does not include. `rustc` 1.83 defaults to edition
2015, so fin3 needs `--edition 2021`. The source is unchanged.

1. **Re-create a journal.** For example:
   ```
   ENDER=1 PBITS=27 NBITS=23 PJOURNAL=j.bin ./fin3 16 38 68 72 257
   ```
   That needs about 2.3 GB of RAM and about 3–10 CPU-hours per case.
   The P-set is order-dependent, so a fresh run may produce a different,
   equally valid Q. To check *our* Q, get the journal with the sha256
   listed in `certificates/LARGE_ARTIFACTS.tsv`. Those files are not in
   git. They are available on request.
2. **Check it with both checkers**, using ENDER=1 or ENDER=0 (see
   METHODS). For our journals, the counts must equal those in
   `certificates/reply38/*_cert_*.txt`.
3. **Checker self-tests:** `code/tests/tests.sh`, `tests_lm.sh` and
   `tests2.sh`. They need the small test journals (fin3 runs on
   {11,16,58}, {16,23,36}, {16,17,90}, {13,16,140}, {16,21,156}), plus
   `mkneg.py` for the forged inputs.

## Status legend

| Label | Meaning |
|---|---|
| **TWO-CHECKER CERTIFIED, ender-free** | Two independent checkers verified a closed P-set with no theorem beyond the game rules |
| **TWO-CHECKER CERTIFIED, conditional on the ender theorem** | Two independent checkers agree; some children are accepted as N by the Hutchings/Sicherman ender theorem (counted and logged) |
| **ONE CHECKER** | One certificate checker plus the solver; the second checker has not run |
| **SINGLE-METHOD SEARCH (unverified)** | One solver's verdict; no certificate, no second method |
| **TIMEOUT/UNKNOWN** | The run hit its limit; no value. Never read as N |
| **NOT RUN (error)** | The solver crashed before any result; no claim |
| **INDEPENDENT CONFIRMATION / CONFIRMED (two engines)** | Our independent engines reproduce an already-published answer |
| **PROVED (short argument)** | A short proof from the Quiet End Theorem, written out in the text |
| **OPEN** | Unresolved |

## Not in this tree

- The source package's `REVIEW_NOTES.md` is internal and is not committed.
  `certificates/MANIFEST.sha256` is the manifest of that source package,
  so it still lists `REVIEW_NOTES.md` (sha256 `b6a14c7c…`) and the
  pre-placement `README.md`. Open questions from those notes are in the
  pull request, not in this folder.
- No compiled binaries. The binaries used for the recorded runs are
  listed by sha256 in `certificates/BINARIES.tsv`.
- No file here is over 5 MB. The P-journals and the two ender-free Q'
  files are listed by size and sha256 in `certificates/LARGE_ARTIFACTS.tsv`.
  The seven fin3 journals sum to 5,942,686,944 bytes. The two Q' files
  sum to 1,625,661,504 bytes. Together that is 7,568,348,448 bytes. The
  table also lists a receipt copy of the 100+245 journal (1,105,134,208
  bytes, a different file hash from the fin3 journal of the same size).
  They are available on request.
- The `.bin` files that are committed are data: ender-key logs (16-byte
  records) and the small checker-test journals that came in the package.
  They are not executables.
- Three journals the checker suites name were left out of the source
  package because they were over its 200 KB cutoff: `j_16_23_36.bin`,
  `j_16_17_90.bin`, and `j_16_21_156.bin`. The recorded suite output
  gives `|Q|` 103,810, 98,890 and 619,755. With the 64-byte header and
  16-byte records, those journals are 1,661,024 bytes, 1,582,304 bytes
  and 9,916,144 bytes. The last is over 5 MB, so it is not added here.
  Regenerate all three with fin3 (see METHODS) before running
  `code/tests/tests.sh`, `tests_lm.sh` and `tests2.sh`. The scripts
  still use the absolute paths of the machine they ran on.

## Credits

Built on the work of Rudi Cilibrasi and collaborators (the upstream
campaign), George Sicherman and Thomas Blok. Hutchings' ender theorem and
the Quiet End Theorem are as stated in Sicherman, Integers 2 (2002) G2.
