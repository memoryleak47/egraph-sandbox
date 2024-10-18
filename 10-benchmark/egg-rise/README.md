## Dependencies

`cargo`, `bash`, `systemd-run`, and GNU `time` for benchmarking

## Run Benchmarks

`./bench.sh`

## Collect Results

`./collect_results.sh > results.csv`

## Preview Results

`cat results.csv | sed -e 's/,,/, ,/g' | column -s, -t | less -#5 -N -S`