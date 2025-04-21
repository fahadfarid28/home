# just manual: https://github.com/casey/just

serve *args:
    #!/bin/sh
    export RUST_BACKTRACE=0
    cargo run -- serve {{args}}

serve-release *args:
    #!/bin/sh
    export RUST_BACKTRACE=0
    cargo run --release -- serve {{args}}

repack:
    beardist build
    ./repack.sh
