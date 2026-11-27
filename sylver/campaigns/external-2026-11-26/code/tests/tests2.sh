#!/bin/bash
# checkq test suite. expect: OK (exit 0), FAIL (exit 1), ERROR (exit 2)
cd /workspace/sylver/certq; C2=/workspace/sylver/certq2/checkq2; pass=0; bad=0
t(){ exp=$1; shift; out=$("$@" 2>/dev/null); rc=$?; got=$( [ $rc -eq 0 ] && echo OK || ([ $rc -eq 1 ] && echo FAIL || echo ERROR) )
  if [ "$got" = "$exp" ]; then pass=$((pass+1)); r=pass; else bad=$((bad+1)); r=WRONG; fi; echo "$r expect=$exp got=$got :: $* :: $out"; }
for c in "11 16 58" "16 23 36" "16 17 90" "13 16 140" "16 21 156"; do t OK $C2 j_${c// /_}.bin $c; done
t OK    $C2 pos_terminal.bin 11 16 58
t OK    $C2 pos_dup.bin 11 16 58
t FAIL  $C2 neg_addN.bin 11 16 58
t FAIL  $C2 neg_noroot.bin 11 16 58
t FAIL  $C2 neg_drop.bin 11 16 58
t FAIL  $C2 neg_badkunz.bin 11 16 58
t FAIL  $C2 neg_noroot_contain.bin 11 16 58
t FAIL  $C2 neg_termchild.bin 11 16 58
t FAIL  $C2 neg_flip.bin 11 16 58
t FAIL  $C2 j_5_16.bin 5 16
t FAIL  $C2 j_11_16.bin 11 16
t FAIL  $C2 neg_Nroot_5_16.bin 5 16
t FAIL  $C2 neg_Nroot_11_16.bin 11 16
t FAIL  env ENDER=1 $C2 neg_Nroot_11_16.bin 11 16
t ERROR $C2 j_11_16_58.bin 11 16 59
t ERROR $C2 j_11_16_58.bin 16 26
t ERROR $C2 j_11_16_58.bin 11 16
t ERROR $C2 /nonexistent 11 16 58
# checkpoint/resume equivalence: kill-free variant: two ranges via CKPT
rm -f /tmp/ck2_test; CKPT=/tmp/ck2_test CKSTEP=20000 STOPAFTER=40000 $C2 j_16_23_36.bin 16 23 36 >/dev/null 2>&1; t OK env CKPT=/tmp/ck2_test CKSTEP=20000 $C2 j_16_23_36.bin 16 23 36
echo "passed=$pass wrong=$bad"
