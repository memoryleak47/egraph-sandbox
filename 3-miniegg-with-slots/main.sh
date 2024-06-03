cargo b --release

function run() {
    echo Running $*
    /usr/bin/time -f "%E, %M Kbytes" timeout -v 5m ./target/release/miniegg-with-slots $*

    echo 
}

run reduction
run fission
run binomial

run reduction --extraction
run fission --extraction
run binomial --extraction
