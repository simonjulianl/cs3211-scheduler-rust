 #!/bin/bash

for i in $(ls implementations/); do
  echo ""
  echo "========== ${i} =========="
  cp implementations/${i} src/main.rs 
  cargo run -r 5664168989938163334 
  # cargo run -r 1976915708242608314 
  # cargo run -r 12605174704058567923
  # cargo flamegraph -o logs/${i}.svg -- 5664168989938163334
  # cargo flamegraph -o logs/${i}.svg -- 1976915708242608314 
  # cargo flamegraph -o logs/${i}.svg -- 12605174704058567923 
  # rm perf.data
done
