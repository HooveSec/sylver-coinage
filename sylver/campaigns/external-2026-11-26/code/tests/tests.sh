#!/bin/bash
# checkq test suite. expect: OK (exit 0), FAIL (exit 1), ERROR (exit 2)
cd /workspace/sylver/certq; pass=0; bad=0
t(){ exp=$1; shift; out=$("$@" 2>/dev/null); rc=$?; got=$( [ $rc -eq 0 ] && echo OK || ([ $rc -eq 1 ] && echo FAIL || echo ERROR) )
  if [ "$got" = "$exp" ]; then pass=$((pass+1)); r=pass; else bad=$((bad+1)); r=WRONG; fi; echo "$r expect=$exp got=$got :: $* :: $out"; }
for c in "11 16 58" "16 23 36" "16 17 90" "13 16 140" "16 21 156"; do t OK ./checkq j_${c// /_}.bin $c; done
t OK    ./checkq pos_terminal.bin 11 16 58
t OK    ./checkq pos_dup.bin 11 16 58
t FAIL  ./checkq neg_addN.bin 11 16 58
t FAIL  ./checkq neg_noroot.bin 11 16 58
t FAIL  ./checkq neg_drop.bin 11 16 58
t FAIL  ./checkq neg_badkunz.bin 11 16 58
t FAIL  ./checkq neg_noroot_contain.bin 11 16 58
t FAIL  ./checkq neg_termchild.bin 11 16 58
t FAIL  ./checkq neg_flip.bin 11 16 58
t FAIL  ./checkq j_5_16.bin 5 16
t FAIL  ./checkq j_11_16.bin 11 16
t FAIL  ./checkq neg_Nroot_5_16.bin 5 16
t FAIL  ./checkq neg_Nroot_11_16.bin 11 16
t FAIL  env ENDER=1 ./checkq neg_Nroot_11_16.bin 11 16
t ERROR ./checkq j_11_16_58.bin 11 16 59
t ERROR ./checkq j_11_16_58.bin 16 26
t ERROR ./checkq j_11_16_58.bin 11 16
t ERROR ./checkq /nonexistent 11 16 58
# checkpoint/resume equivalence: kill-free variant: two ranges via CKPT
rm -f ck_test; CKPT=ck_test CKSTEP=20000 END=50000 ./checkq j_16_23_36.bin 16 23 36 >/dev/null 2>&1; t OK env CKPT=ck_test CKSTEP=20000 ./checkq j_16_23_36.bin 16 23 36
echo "passed=$pass wrong=$bad"
