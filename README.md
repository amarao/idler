# Idler

Idler is a debugging utility to create a high-load condition on the host
without actual need to high-volume traffic.

Currently it establishes ~512k connections to itself (totalling to
1M open network sockets) and send one byte every 10 seconds.

On my machine this is corresponds to ~145% CPU utilization (1.5 cores), 62%
sys utilization and 43% of IRQ), ~80Mbps on lo.

# How to run

To run it install Rust toolchain and run `cargo run --release`. Sorry, no
binary packages for now.

# Issues

* You may want to raise open file limit (`ulimit -n 1048576`).
* Your dmesg may become full of `nf_conntrack: nf_conntrack: table full, dropping packet`
  (remove conntrack from localhost).
* `TCP: request_sock_TCP: Possible SYN flooding on port 127.0.0.1:6379. Sending cookies.`
  (should be okay to ignore).
* While program is establishing connections, everything will be sluggish on the computer
  (including unrelated tasks).

# Code

I'm learning Rust (tokyo and async, specifically), so my code is amateur.

Reasonable critique and advices are welcomed.
