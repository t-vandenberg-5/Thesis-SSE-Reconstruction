# SSE Reconstruction
This repository contains the code for my master's thesis.

## Experiments
To run the experiments, I used the `src/bin/reconstruct.rs` binary, while
changing the configuration, such as the `MAX` constant or `TrackerD` type. This
then reconstructs generated databases and writes output to stdout.

For the real-world data experiments I extracted the dataset on my machine, and
provided it through stdin to `src/bin/spitz-{midline,full}-{fast,3d,4d,5d}.rs`.
These binaries would then use 25 threads to run 100 reconstructions and print
the p50, p90, and p99 info to stdout on top of other logging.

## Plotting
To create the charts I used, I formatted the results from
`src/bin/reconstruct.rs` by hand into CSV files, which I used to create charts
in `src/bin/plot.rs`. 
