#!/bin/bash
# Second-method queue (Sylver theory worker). Sequential, 1 core, nice 15, 2 GB address-space cap per job.
cd /workspace/sylver/theory; ulimit -v 2000000
LOG=second_method.log
while pgrep -f ender_ab.sh >/dev/null; do sleep 10; done
j(){ name=$1; shift; grep -q "^DONE $name " $LOG 2>/dev/null && return; echo "$(date '+%F %T %Z') START $name: $*" >> $LOG
  s=$(date +%s); out=$(timeout 7200 nice -n 15 "$@" 2>&1); rc=$?; e=$(( $(date +%s)-s ))
  echo "$out" | tail -3 | sed "s/^/  /" >> $LOG; echo "DONE $name rc=$rc ${e}s" >> $LOG; }
# Q1: replay Cilibrasi o16 ledger finite witnesses (expected P) with our two finite engines
for p in "16 23 36" "11 16 58" "16 17 90" "13 16 140" "16 21 156"; do
  k=$(echo $p|tr ' ' _); j "solve_$k" ../solve $p; j "vchkf_$k" env VMB=1200 ../verify/vchk -f $p; done
# Q2: g=2 nodes behind {16,14},{16,20},{16,22},{16,24} answers, via vchk (QET, no lemmas)
for p in "8 14" "12 16 22" "16 20 34" "10 16 24" "16 20 46"; do k=$(echo $p|tr ' ' _); j "vchk_$k" env VMB=1200 ../verify/vchk $p; done
echo "$(date '+%F %T %Z') QUEUE COMPLETE" >> $LOG
