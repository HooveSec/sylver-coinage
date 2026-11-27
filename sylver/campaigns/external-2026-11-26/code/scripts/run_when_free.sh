#!/bin/bash
# Queue checkq ENDER=1 for {16,38,72,100,245}. Waits until checkq2 no-theorem on Q' is done (or not running) AND MemAvailable >= 5 GB.
# Raises ulimit to 6.5 GB (69M keys need ~5.4 GB with checkq's 2x table). One instance. Sylver worker 2026-10-19 11:50 EDT.
cd /workspace/sylver/certq/q100; exec 9>.gate.lock; flock -n 9 || exit 0
# already finished (OK/FAIL) — do not block resume_all waiting for 5 GB
if grep -qE '^(OK|FAIL)' /workspace/sylver/certq/cert_16_38_72_100_245.txt 2>/dev/null; then
  echo "$(date '+%F %T %Z') run_when_free: cert_16_38_72_100_245 already done, exit" >> gate.log
  exit 0
fi
R2=/workspace/sylver/certq2/cert_16_38_68_72_257_e0.txt
while ! grep -qE '^(OK|FAIL|ERROR)' $R2 2>/dev/null; do
  # also proceed if checkq2e0 process is gone AND a result exists, or after long wait if result and no process
  pgrep -f 'checkq2 .*qprime_16_38_68_72_257' >/dev/null || { [ -f $R2 ] && break; }
  sleep 120
done
# wait for 5 GB available (pause S88 is memguard's job; we just wait)
while true; do
  av=$(awk '/MemAvailable/{print int($2/1024)}' /proc/meminfo)
  [ $av -ge 5120 ] && break
  echo "$(date '+%F %T %Z') waiting for 5 GB avail (now ${av}MB)" >> gate.log
  sleep 60
done
echo "$(date '+%F %T %Z') start cert 100+245 avail=${av}MB" >> gate.log
# extract Q (chunked, ~200 MB) if not already done
[ -f q_38_72_100_245.bin.sha256 ] || nice -n 15 python3 extract_q.py /workspace/sylver/fin3/j_38_72_100_245.bin q_38_72_100_245.bin 1500000 >> gate.log 2>&1
# checkq wants fin3 journal format (16-B records); extracted file is that. Point run_cert at it via symlink in the usual place? run_cert looks at fin3/j_*.bin.
# Use the original journal (already unique enough); extract is for the receipt. Raise ulimit for this case.
cd /workspace/sylver/certq
grep -qE '^(OK|FAIL)' cert_16_38_72_100_245.txt 2>/dev/null && exit 0
exec 8>.cert_16_38_72_100_245.lock; flock -n 8 || exit 0
echo "$(date '+%F %T %Z') start checkq 16 38 72 100 245 sha_checkq=$(sha256sum checkq | cut -c1-16)" >> cert_16_38_72_100_245.log
( ulimit -v 6500000; ENDER=1 ENDERLOG=enders_16_38_72_100_245.bin CKPT=ck_16_38_72_100_245 CKSTEP=1000000 nice -n 10 ./checkq /workspace/sylver/fin3/j_38_72_100_245.bin 16 38 72 100 245 >> cert_16_38_72_100_245.txt 2>> cert_16_38_72_100_245.log )
rc=$?; echo "$(date '+%F %T %Z') end rc=$rc: $(tail -1 cert_16_38_72_100_245.txt)" >> cert_16_38_72_100_245.log
# plan note for theorem-free extension once this OK
echo "$(date '+%F %T %Z') next: ENDER=0 solve logged enders (same as 68+257 ender0/), merge into Q', re-check ENDER=0" >> /workspace/sylver/certq/q100/gate.log
exit $rc
