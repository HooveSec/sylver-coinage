# External checks, 19 October 2026

Independent checks of records already in this repository, plus two notes the
records do not contain. Opening 16 remains unresolved. No priority is claimed.

The checks were made with solvers outside this repository: a finite Rust
solver, a finite C checker, and a g = 2 C checker with exact pattern cycles.
This directory's `audit.py` replays only the small finite leaves, using the
Python solver here.

## Answers already in the ledger

The certified table answers 14 by 8, 22 by 12, and 24 by 10. A second method
(Quiet End Theorem, plus exact cycles for the odd moves of `{8,10,12,14}`,
`{8,10,22}`, `{8,12,18,22}`, `{8,12,26,30}`, and `{8,12,34,38}`, plus even
replies down to `{2,3}`, `{4,6}`, or `{8,10,14,19}`) agrees:

| Reply | Answer | Position reached | This check |
| ---: | ---: | --- | --- |
| 14 | 8 | `{8,14}` | P, 1913 states in the C checker |
| 22 | 12 | `{12,16,22}` | P, 26043 states |
| 24 | 10 | `{10,16,24}` | P, 10535 states |

`{8,10,22}` repeats from 49 with period 8, as already stated in this
repository.

The same method shows `{16,20,46}` is P (7160704 states), so 46 answers 20.
`{16,20,46}` is on [Sicherman's list of P-positions](https://sicherman.net/sylver/ppos.html),
and the reply-40 record already certifies it as Q. It is not a row of
`OPENING_16_EVEN_RESPONSES`. The certified answer to 20 remains 34, through
T. This method does not prove T: `{16,20,34}` stays unknown with these
lemmas.

The finite ledger rows were replayed from empty memos. Both outside engines
report P at the Frobenius numbers already in the certificates. State counts
differ and are not proposed as corrections.

| Certificate | Destination | Frobenius | Outside outcome |
| --- | --- | ---: | --- |
| `o36-certificate.json` | `{16,23,36}` | 145 | P |
| `o58-certificate.json` | `{11,16,58}` | 111 | P |
| `o90-certificate.json` | `{16,17,90}` | 159 | P |
| `o140-certificate.json` | `{13,16,140}` | 163 | P |
| `o156-certificate.json` | `{16,21,156}` | 215 | P |

## The reply 40

`python -m sylver.analyze_opening_16 40` reports no exceptional odd winning
move. That fact is not in the campaign records.

`{16,40} = 8·{2,5}`. The gaps of `<2,5>` are 1 and 3. For every odd `z ≥ 5`,
`{16,40,z}` is a quiet ender of Frobenius number `7z+24 > 1`, so it is N.
`{16,40,3} = {3,16}`, and the move 2 leaves `{2,3}`, which is P
([proofs/3-16.json](proofs/3-16.json)). Naming 1 loses. So 40 has no odd
answer. Any answer is even. 40 is not solved. 38 remains the lowest reply
without a certified answer; by search, 72 answers 38 and is not certified.
40 has no answer in the ledger, certified or otherwise.

## Blok, 14-even, §5.2

In *Sylver Coinage even positions in 14* (2022), the cell for
`{14,22,24,32,34,40}` is 251. It should not be read as a winning move.

- 25 reaches a P-position, Frobenius number 55
  ([proofs/blok14-25.json](proofs/blok14-25.json)).
- 251 is N, by the reply 25. The child is the same P-position, because 251
  is larger than 55 ([proofs/blok14-251.json](proofs/blok14-251.json)).

Odd moves from 3 through 401 have no other win. Even moves were not
searched. The cell does not affect the opening 16.

## Reproduction

From the repository root, the small part:

```sh
python sylver/campaigns/external-2026-10-19/audit.py
python -m sylver.analyze_opening_16 40
```

The large replays are not repeated by that audit.
