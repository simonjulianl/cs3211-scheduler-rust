 #!/bin/bash

for i in $(ls implementations/); do
  echo ""
  echo "========== ${i} =========="
  cp implementations/${i} src/main.rs 
  # cargo run -r 5664168989938163334 6 6; cargo run -r 1976915708242608314 6 6; cargo run -r 12605174704058567923 6 6
  cargo run -r 5664168989938163334; cargo run -r 1976915708242608314; cargo run -r 12605174704058567923
done