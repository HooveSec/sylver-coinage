#!/bin/bash
# A/B: solve (plain) vs solve_e (ender-pruned): values must agree; compare node counts. 1 core, 2 GB cap.
ulimit -v 2000000
out=ender_ab.tsv; : > $out
run(){ a=$(timeout 300 nice -n 10 ../solve "$@" 2>&1); b=$(timeout 300 nice -n 10 ./solve_e "$@" 2>&1)
  va=$(echo "$a"|grep -oE ' (N|P) nodes'|cut -d' ' -f2); vb=$(echo "$b"|grep -oE ' (N|P) nodes'|cut -d' ' -f2)
  na=$(echo "$a"|grep -oE 'nodes=[0-9]+'|cut -d= -f2); nb=$(echo "$b"|grep -oE 'nodes=[0-9]+'|cut -d= -f2)
  ta=$(echo "$a"|grep -oE '[0-9.]+s$'); tb=$(echo "$b"|grep -oE '[0-9.]+s$')
  echo -e "$*\t$va\t$vb\t$na\t$nb\t$ta\t$tb" >> $out; }
for x in 3 5 7 9 11 13 15 17 19; do run 16 $x; done
for m in 25 251; do run 14 22 24 32 34 40 $m; done
tail -n +2 ../verify/blok_fin.tsv | cut -f1 | tr ',' ' ' | while read p; do run $p; done
echo DONE >> $out
