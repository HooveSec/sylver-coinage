#!/bin/bash
cd /workspace/sylver/verify; ulimit -v 1500000
echo "$(date "+%F %T %Z") run_v start pid=$$" >> v_even.err
for N in 6 10 14 18 22; do grep -q "^G2 {16,$N}" v_even.txt 2>/dev/null || echo "$(date "+%F %T %Z") vchk 16 $N start" >> v_even.err; grep -q "^G2 {16,$N}" v_even.txt 2>/dev/null && continue; nice -n 10 env VMB=800 ./vchk 16 $N >> v_even.txt 2>>v_even.err; done
