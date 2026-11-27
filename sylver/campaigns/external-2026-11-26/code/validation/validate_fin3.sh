#!/bin/bash
# fin3 vs theory/ender_ab.tsv (values from ../solve; 383 finite positions: {16,odd}, Blok typo pair, 372 Blok 6-16 finite rows)
cd /workspace/sylver/fin3; : > validate_fin3.tsv
while IFS=$'\t' read -r pos a b rest; do
  for E in 1 0; do
    out=$(PBITS=22 NBITS=20 ENDER=$E timeout 60 taskset -c ${CPU:-2} ./fin3 $pos 2>/dev/null); v=$(echo "$out" | grep -oE ' (P|N) nodes' | tr -d ' a-z=')
    [ -z "$v" ] && v=TIMEOUT; echo -e "$pos\t$a\tE$E\t$v" >> validate_fin3.tsv
  done
done < /workspace/sylver/theory/ender_ab.tsv
awk -F'\t' '{n++; if($4=="TIMEOUT")t++; else if($2==$4)ok++; else bad++} END{print "fin3 validation: runs="n" agree="ok" CONTRA="bad+0" timeout="t+0}' validate_fin3.tsv | tee validate_fin3.summary
