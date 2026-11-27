#!/bin/bash
# Independent replays of Cilibrasi's 7 search-only witnesses for {16,38,72} (each claimed P) with fin3 (P-table + journal).
# Resumable: each case has its own journal j_<case>.bin; finished cases are listed in results38.txt. One instance (flock).
cd /workspace/sylver/fin3
exec 9>/workspace/sylver/fin3/.run38.lock; flock -n 9 || { echo "run38 already running"; exit 0; }
[ -f VALIDATED ] || { echo "$(date '+%F %T %Z') run38: fin3 not validated, refusing" >> /workspace/sylver/progress.log; exit 1; }
# order: 68+257 first (smallest P-table), then the other P-table witnesses, then the memo ones
for c in "38 68 72 257" "38 72 100 245" "38 72 78 367" "38 72 116 229" "38 72 84 139" "38 72 119 154" "38 72 106 137"; do
  [ -n "$ONLY" ] && [ "$c" != "$ONLY" ] && continue
  tag=$(echo $c | tr ' ' '_'); grep -q "^\[16, ${c// /, }\]" results38.txt 2>/dev/null && continue
  echo "$(date '+%F %T %Z') fin3 start {16,${c// /,}}" >> /workspace/sylver/progress.log
  ( ulimit -v 2900000; PBITS=27 NBITS=23 ENDER=1 RATE_SEC=600 PJOURNAL=/workspace/sylver/fin3/j_$tag.bin nice -n 5 ./fin3 16 $c >> results38.txt 2>> log_$tag.err )
  rc=$?; echo "$(date '+%F %T %Z') fin3 end {16,${c// /,}} rc=$rc: $(tail -1 results38.txt)" >> /workspace/sylver/progress.log
  [ $rc -eq 0 ] || exit $rc
done
