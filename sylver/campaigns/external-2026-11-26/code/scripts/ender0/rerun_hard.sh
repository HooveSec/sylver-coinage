#!/bin/bash
# Re-solve ender children that timed out (rc!=0) with a bigger P-table (PBITS=25, ~0.6 GB), no time limit, resuming from their P-journal;
# then replace their results.tsv row and start run_final_e0.sh again. Sylver worker 2026-10-19 07:40 EDT.
cd /workspace/sylver/certq/ender0; exec 9>.hard.lock; flock -n 9 || exit 0
grep -v ' N nodes' results.tsv | while IFS=$'\t' read hex gens rest; do
  t0=$(date +%s); out=$( ( ulimit -v 1200000; PBITS=25 NBITS=22 ENDER=0 RATE_SEC=600 PJOURNAL=j_$hex.bin nice -n 10 ../../fin3/fin3 $gens ) 2>>err_$hex.txt ); rc=$?
  printf "%s\t%s\trc=%s\t%ss\t%s\n" "$hex" "$gens" $rc $(( $(date +%s)-t0 )) "$out" >> results_hard.tsv
  [ $rc -eq 0 ] && { grep -v "^$hex	" results.tsv > results.tmp; printf "%s\t%s\trc=%s\t%ss\t%s\n" "$hex" "$gens" $rc $(( $(date +%s)-t0 )) "$out" >> results.tmp; mv results.tmp results.tsv; }
done
echo "$(date '+%F %T %Z') rerun_hard done" >> results.log
nohup setsid ./run_final_e0.sh > final.out 2>&1 < /dev/null &
