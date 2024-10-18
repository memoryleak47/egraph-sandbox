#!/bin/bash

set -e
cargo b --release
mkdir outputs -p
rm -f outputs/*

function run() {
    (
        echo $1 $2 $3
        echo ----------------
        /usr/bin/time -f "%E, %M Kbytes" timeout -v 5m ./target/release/egg-rise $1 $2 $3
    ) |& tee outputs/$1_$2_$3.txt

    echo
    echo
    echo
}

binding=name

run reduction $binding
run fission $binding
run binomial $binding

run reduction $binding eta-exp
run fission $binding eta-exp
run binomial $binding eta-exp
