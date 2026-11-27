#!/bin/bash
# Full P-set certificate check of a fin3 journal (checkq v3), 1 core, niced, <= 2.5 GB, checkpointed every 1M keys, resumable. One per case (flock).
# usage: run_cert.sh 16 38 68 72 257
cd /workspace/sylver/certq; tag=$(echo "$@" | tr ' ' _); J=/workspace/sylver/fin3/j_${tag#16_}.bin; [ -f $J ] || J=/workspace/sylver/fin3/j_$tag.bin
exec 9>/workspace/sylver/certq/.cert_$tag.lock; flock -n 9 || { echo "cert $tag already running"; exit 0; }
grep -qE '^(OK|FAIL)' cert_$tag.txt 2>/dev/null && exit 0
echo "$(date '+%F %T %Z') start checkq $* journal=$J sha_checkq=$(sha256sum checkq | cut -c1-16)" >> cert_$tag.log
( ulimit -v 2500000; ENDER=1 ENDERLOG=enders_$tag.bin CKPT=ck_$tag CKSTEP=1000000 nice -n 10 ./checkq $J "$@" >> cert_$tag.txt 2>> cert_$tag.log )
rc=$?; echo "$(date '+%F %T %Z') end rc=$rc: $(tail -1 cert_$tag.txt)" >> cert_$tag.log; exit $rc
