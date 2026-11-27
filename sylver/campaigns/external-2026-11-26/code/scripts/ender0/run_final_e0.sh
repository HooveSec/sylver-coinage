#!/bin/bash
# Final ender-free re-check (Sylver worker 2026-10-19). Waits for laneCERT (checkq ENDER=1) to finish and laneE0 to solve every logged ender child,
# then builds Q' = Q u ender0 journals (merge.py) and checks it with checkq ENDER=0, then (after laneCERT2 ends, memory) checkq2 ENDER=0.
# Results: certq/ender0/final_checkq_e0.txt, final_checkq2_e0.txt. One instance (flock). Nothing else is touched.
cd /workspace/sylver/certq/ender0; exec 9>.final.lock; flock -n 9 || exit 0
R=/workspace/sylver/certq/cert_16_38_68_72_257.txt
until grep -qE '^(OK|FAIL)' $R 2>/dev/null && grep -q 'all ender children done' results.log 2>/dev/null; do sleep 120; done
grep -q '^OK' $R || { echo "$(date '+%F %T %Z') checkq ENDER=1 did not pass; final step skipped" >> results.log; exit 1; }
if grep -v ' N nodes' results.tsv | grep -q .; then echo "$(date '+%F %T %Z') some ender child not solved N under ENDER=0; stop" >> results.log; exit 1; fi
[ -f qprime_16_38_68_72_257.bin ] || python3 merge.py qprime_16_38_68_72_257.bin >> results.log 2>&1
if ! grep -qE '^(OK|FAIL)' final_checkq_e0.txt 2>/dev/null; then
  ( ulimit -v 2500000; ENDER=0 CKPT=ck_final_checkq_e0 CKSTEP=1000000 nice -n 10 ../checkq qprime_16_38_68_72_257.bin 16 38 68 72 257 >> final_checkq_e0.txt 2>> final_checkq_e0.log ); fi
echo "$(date '+%F %T %Z') checkq ENDER=0 on Q': $(tail -n 1 final_checkq_e0.txt)" >> results.log
until grep -qE '^(OK|FAIL)' /workspace/sylver/certq2/cert_16_38_68_72_257.txt 2>/dev/null; do sleep 120; done
ENDER=0 J=/workspace/sylver/certq/ender0/qprime_16_38_68_72_257.bin TAG=_e0 /workspace/sylver/certq2/run_cert2.sh 16 38 68 72 257
echo "$(date '+%F %T %Z') checkq2 ENDER=0 on Q': $(tail -n 1 /workspace/sylver/certq2/cert_16_38_68_72_257_e0.txt)" >> results.log
