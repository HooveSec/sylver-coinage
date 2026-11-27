#!/bin/bash
# Frontier lane (after the {16,26,88} check on core 5). Each stage stops at its first P child (= an answer). Resumable.
# 40: even a = 2 mod 4 only (no odd answer exists, theory worker/QET). 44: the 9 odd candidates (odd gaps of {4,11} > 1), then even.
# 42, 46 (g=2): odd candidates first (odd gaps of {8,21}/{8,23} > 1, fin3), then all even gaps.
cd /workspace/sylver/g4lane
exec 9>/workspace/sylver/g4lane/.lock; flock -n 9 || { echo "g4lane already running"; exit 0; }
[ -f /workspace/sylver/even3/VALIDATED ] && [ -f /workspace/sylver/fin3/VALIDATED ] || exit 1
export TRY_TO=${TRY_TO:-10800} EMAX=${EMAX:-250}
odd_gaps(){ python3 -c "
import sys;a,b=map(int,sys.argv[1:]);S={0}
for n in range(1,a*b):
  if (n>=a and n-a in S) or (n>=b and n-b in S): S.add(n)
print(','.join(str(n) for n in range(3,a*b,2) if n not in S))" $1 $2; }
run(){ T="$1"; shift; grep -qx "$T" g4lane.done 2>/dev/null && return 0
  env "$@" nice -n 5 taskset -c ${G4CPU:-5} ./g4lane.py $T >> g4lane.jsonl 2>> g4lane.err; rc=$?
  [ $rc -eq 0 ] && echo "$T" >> g4lane.done || { echo "$(date '+%F %T %Z') g4lane {${T// /,}} rc=$rc" >> /workspace/sylver/progress.log; exit $rc; }; }
run "16 40"
run "16 44" ODDS=$(odd_gaps 4 11)
run "16 42" ODDS=$(odd_gaps 8 21)
run "16 46" ODDS=$(odd_gaps 8 23)
