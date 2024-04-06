# On-Site Rust Channel Benchmarking Helper

Deploy on server to determine which public crates are the fastest for communicating in different architectures

## Analysis Pipeline
1. `cargo run --release`
2. Once the experiments are completed, run `python3 analyze.py` to view the results and the corresponding latency distributions

## Crates tested by Architecture:
### Async
- Kanal (https://crates.io/crates/kanal)
- Flume (https://crates.io/crates/flume)
- Tokio (https://crates.io/crates/tokio)
- Async_Channel (https://crates.io/crates/async-channel)
- Crossfire (https://crates.io/crates/crossfire)
- Thingbuf (https://crates.io/crates/thingbuf)
- Tachyonix (https://crates.io/crates/tachyonix)
- Postage (https://crates.io/crates/postage)
- Async_Broadcast (https://crates.io/crates/async-broadcast)
### Busy-Spinning
- Rtrb (https://crates.io/crates/rtrb)
- Ringbuf (https://crates.io/crates/ringbuf)
- Crossbeam-channel (https://crates.io/crates/crossbeam-channel)
- Crossbeam-queue (https://crates.io/crates/crossbeam-queue)
- Kanal (https://crates.io/crates/kanal)
- Flume (https://crates.io/crates/flume)
- Omango (https://crates.io/crates/omango)
- Npnc (https://crates.io/crates/npnc)
- Magnetic (https://crates.io/crates/magnetic)
- Tokio (https://crates.io/crates/tokio)


## Future Plans
I'd like to split up the experiments to test `mpmc` / `mpsc` / `spsc` against each other in these different architectures. This will be included at some point but for now `mpmc` & `mpsc` have been favored as there are more use cases for them

## Notes
- Some of these crates (such as `async_broadcast`) are tailored to specific features which incur some speed disadvantages. Thus, I recommend doing adequate research to determine the best crate for your architecture and only use this as a reference.
- Thread-pinning has been included in most of these experiments so please check the `main()` function and change the parameters as needed. If you'd like more realistic non-jittery results, consider using `isolcpus` or the like
- `payload_size` parameter was included in `main()` as well, feel free to change this at your discretion. The default is 0 bytes
 

### P.s.
If you'd like another crate added, have questions / ideas, please DM me on twitter `@Dub0x3A` or on discord `0xdub`. Cheers and thanks