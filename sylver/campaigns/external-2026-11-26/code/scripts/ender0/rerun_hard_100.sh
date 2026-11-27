#!/bin/bash
# Lane E0-100 hard rerun (Sylver worker 2026-10-27 04:15 PDT), modeled on certq/ender0/rerun_hard.sh.
# Re-solve every ender child of {16,38,72,100,245} whose results.tsv row is not "N" (the 3 rc=124 timeouts: their PBITS=22 P-table
# saturated at 3,565,158 = 0.85*2^22 entries) with a bigger P-table (PBITS=25, ~0.6 GB), no time limit, resuming from the child's P-journal.
# Idempotent: one instance (flock), only non-N rows are retried, an N result replaces the row. Each fin3 is ulimit -v 1.2 GB, nice 10,
# and starts only when MemAvailable >= 1.5 GB (box memory budget). Log: results_hard.tsv / results.log / err_<hex>.txt (appended).
# Then starts run_final_e0.sh (itself flock'd and waits for all children N). Certificate check of Cilibrasi's published witness, not a new result.
cd /workspace/sylver/certq/ender0_100; exec 9>.hard.lock; flock -n 9 || exit 0
memwait(){ while [ $(awk '/MemAvailable/{print int($2/1024)}' /proc/meminfo) -lt ${1:-1500} ]; do sleep 60; done; }
grep -v ' N nodes' results.tsv | while IFS=$'\t' read hex gens rest; do
  memwait 1500; echo "$(date '+%F %T %Z') rerun_hard start $hex [$gens] PBITS=25" >> results.log
  t0=$(date +%s); out=$( ( ulimit -v 1200000; PBITS=25 NBITS=22 ENDER=0 RATE_SEC=600 PJOURNAL=j_$hex.bin nice -n 10 ../../fin3/fin3 $gens ) 2>>err_$hex.txt ); rc=$?
  row=$(printf "%s\t%s\trc=%s\t%ss\t%s" "$hex" "$gens" $rc $(( $(date +%s)-t0 )) "$out"); echo "$row" >> results_hard.tsv
  echo "$(date '+%F %T %Z') rerun_hard end $hex rc=$rc $out" >> results.log
  if [ $rc -eq 0 ] && echo "$out" | grep -q ' N nodes'; then grep -v "^$hex	" results.tsv > results.tmp; echo "$row" >> results.tmp; mv results.tmp results.tsv; fi
done
echo "$(date '+%F %T %Z') rerun_hard done; non-N rows left: $(grep -vc ' N nodes' results.tsv)" >> results.log
pgrep -f ender0_100/run_final_e0.sh >/dev/null || nohup setsid /workspace/sylver/certq/ender0_100/run_final_e0.sh > final.out 2>&1 < /dev/null &
