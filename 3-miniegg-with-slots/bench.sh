#!/bin/bash

set -e
cargo b --release
mkdir outputs -p
rm -f outputs/*

function run() {
    (
        echo $1 $2
        echo ----------------
        /usr/bin/time -f "%E, %M Kbytes" timeout -v 5m ./target/release/miniegg-with-slots $1 $2
    ) |& tee outputs/$1_$2.txt

    echo
    echo
    echo
}

run reduction
run fission
run binomial

run reduction eta-exp
run fission eta-exp
run binomial eta-exp
