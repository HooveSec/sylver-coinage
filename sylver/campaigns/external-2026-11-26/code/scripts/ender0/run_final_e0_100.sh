#!/bin/bash
# Final ender-free re-check for the reply-38 witness {16,38,72,100,245} (Sylver worker 2026-10-27 04:30 PDT), modeled on certq/ender0/run_final_e0.sh.
# This replays Cilibrasi's already-published witness: any OK is a certificate check of a known claim, NOT a new Sylver result.
# 1. wait until checkq ENDER=1 has a verdict (OK) and every logged ender child is solved N under fin3 ENDER=0 (stop + log otherwise);
# 2. build Q' = Q u ender0 journals with merge_lm.py (memory-safe; byte-identical to merge.py+dedup_qprime.py on 68+257), ulimit 2.5 GB, needs 3 GB avail;
# 3. checkq ENDER=0 on Q' using checkq_lm (same check code as checkq, static sorted table ~16 B/key = ~1.4 GB instead of checkq's 2^28-slot
#    hash table ~5.2 GB; output byte-identical to checkq on the 24-case tests.sh suite), ulimit 2.5 GB, checkpoint every 1M keys, needs 3 GB avail;
# 4. in parallel, once checkq2 ENDER=1 on Q (PID 14813 at setup) has a verdict and >= 2.5 GB is available: checkq2 ENDER=0 on Q' via
#    certq2/run_cert2.sh TAG=_e0 ULIM=2.0 GB (checkpointed). Results: final_checkq_e0.txt, certq2/cert_16_38_72_100_245_e0.txt; summary in results.log.
# One instance (flock); rerun-safe at every step (skips finished steps, resumes checkpoints).
cd /workspace/sylver/certq/ender0_100; exec 9>.final.lock; flock -n 9 || exit 0
ROOT="16 38 72 100 245"; R=/workspace/sylver/certq/cert_16_38_72_100_245.txt; Q=/workspace/sylver/fin3/j_38_72_100_245.bin
QP=/workspace/sylver/certq/ender0_100/qprime_16_38_72_100_245.bin; R2=/workspace/sylver/certq2/cert_16_38_72_100_245.txt; R2E=/workspace/sylver/certq2/cert_16_38_72_100_245_e0.txt
log(){ echo "$(date '+%F %T %Z') final_e0: $*" >> results.log; }
avail(){ awk '/MemAvailable/{print int($2/1024)}' /proc/meminfo; }
memwait(){ local n=0; while [ $(avail) -lt $1 ]; do [ $((n++ % 30)) -eq 0 ] && log "waiting for MemAvailable >= $1 MB (now $(avail) MB) before $2"; sleep 60; done; }
until grep -qE '^(OK|FAIL)' $R 2>/dev/null && grep -q 'all ender children done' results.log 2>/dev/null; do sleep 120; done
grep -q '^OK' $R || { log "checkq ENDER=1 did not pass; final step skipped"; exit 1; }
exec 8>.hard.lock; flock 8; exec 8>&-   # wait while rerun_hard.sh (holds .hard.lock) is still re-solving children
if grep -v ' N nodes' results.tsv | grep -q .; then log "some ender child not solved N under ENDER=0 (see results.tsv / results_hard.tsv); stop"; exit 1; fi
[ $(wc -l < results.tsv) -eq $(wc -l < list.txt) ] || { log "results.tsv rows $(wc -l < results.tsv) != list.txt children $(wc -l < list.txt); stop"; exit 1; }
if [ ! -f $QP ]; then memwait 3000 "merge_lm"; log "building Q' (avail $(avail) MB)"
  ( ulimit -v 2500000; nice -n 10 python3 merge_lm.py $Q $QP . ) >> results.log 2>&1 || { log "merge_lm failed rc=$?"; exit 1; }
  sha256sum $QP > $QP.sha256; fi
# checkq2 ENDER=0 branch (background, own flock inside run_cert2.sh)
( until grep -qE '^(OK|FAIL)' $R2 2>/dev/null; do sleep 120; done
  if grep -qE '^(OK|FAIL)' $R2E 2>/dev/null; then :; else memwait 2500 "checkq2 ENDER=0"; log "start checkq2 ENDER=0 on Q' (avail $(avail) MB)"
    ENDER=0 J=$QP TAG=_e0 ULIM=2000000 /workspace/sylver/certq2/run_cert2.sh $ROOT; fi
  log "checkq2 ENDER=0 on Q': $(tail -n 1 $R2E)  [checkq2 ENDER=1 on Q: $(tail -n 1 $R2)]" ) &
B=$!
if ! grep -qE '^(OK|FAIL)' final_checkq_e0.txt 2>/dev/null; then memwait 3000 "checkq_lm ENDER=0"; log "start checkq_lm ENDER=0 on Q' (avail $(avail) MB)"
  ( ulimit -v 2500000; ENDER=0 CKPT=ck_final_checkq_e0 CKSTEP=1000000 nice -n 10 ../checkq_lm $QP $ROOT >> final_checkq_e0.txt 2>> final_checkq_e0.log ); fi
log "checkq(_lm) ENDER=0 on Q': $(tail -n 1 final_checkq_e0.txt)"
wait $B
log "done (certificate check of Cilibrasi's published reply-38 witness 72 via {16,38,72,100,245}; not a new result)"
