#!/bin/bash
# Second, independent full P-set check (checkq2, Rust, Apery-vector moves, static CSR table) of a fin3 journal. 1 thread, nice 10,
# ulimit 1.5 GB (measured RSS 571 MB on 68+257), checkpoint every 1M keys (certq2/ck_<case>), result certq2/cert_<case>.txt. One per case.
# usage: run_cert2.sh 16 38 68 72 257 (ENDER=1 by default, logged; pass ENDER=0 / J=journal / TAG=suffix for the final ender-free re-check)
cd /workspace/sylver/certq2; tag=$(echo "$@" | tr ' ' _)${TAG}; J=${J:-/workspace/sylver/fin3/j_$(echo "$@" | cut -d' ' -f2- | tr ' ' _).bin}
exec 9>.cert2_$tag.lock; flock -n 9 || { echo "cert2 $tag already running"; exit 0; }
grep -qE '^(OK|FAIL)' cert_$tag.txt 2>/dev/null && exit 0
echo "$(date '+%F %T %Z') start checkq2 $* journal=$J ender=${ENDER:-1} sha_checkq2=$(sha256sum checkq2 | cut -c1-16)" >> cert_$tag.log
( ulimit -v ${ULIM:-1500000}; ENDER=${ENDER:-1} ENDERLOG=enders_$tag.bin CKPT=ck_$tag CKSTEP=1000000 THREADS=1 nice -n 10 ./checkq2 $J "$@" >> cert_$tag.txt 2>> cert_$tag.log )
rc=$?; echo "$(date '+%F %T %Z') end rc=$rc: $(tail -1 cert_$tag.txt)" >> cert_$tag.log; exit $rc
