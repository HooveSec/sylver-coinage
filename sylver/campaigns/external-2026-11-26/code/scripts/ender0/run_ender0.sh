#!/bin/bash
# Remove the ender dependency (Sylver worker 2026-10-19): solve every distinct ender child Y logged by checkq ENDER=1 with fin3 ENDER=0
# (no theorems, only t<=1 => P), one at a time, nice 15, <= 1.2 GB. Each P-journal j_<hex>.bin holds P positions of Y's game, including the
# winning reply Y+y, so Q' = Q u (all journals, re-keyed mod 16) can be re-checked with ENDER=0 at the end. Results: results.tsv. One instance.
cd /workspace/sylver/certq/ender0; exec 9>.lock; flock -n 9 || exit 0
LOG=../enders_16_38_68_72_257.bin
while true; do
  python3 gens.py $LOG > list.txt
  todo=0
  while read hex gens; do grep -q "^$hex	" results.tsv 2>/dev/null && continue; todo=1
    t0=$(date +%s); out=$( ( ulimit -v 1200000; PBITS=22 NBITS=20 ENDER=0 RATE_SEC=300 PJOURNAL=j_$hex.bin timeout 3600 nice -n 15 ../../fin3/fin3 $gens ) 2>err_$hex.txt ); rc=$?
    printf "%s\t%s\trc=%s\t%ss\t%s\n" "$hex" "$gens" $rc $(( $(date +%s)-t0 )) "$out" >> results.tsv
  done < list.txt
  # stop once the certificate run has finished and every logged child is done
  if [ $todo -eq 0 ] && grep -qE '^(OK|FAIL)' ../cert_16_38_68_72_257.txt 2>/dev/null; then echo "$(date '+%F %T %Z') all ender children done" >> results.log; exit 0; fi
  [ $todo -eq 0 ] && sleep 120
done
