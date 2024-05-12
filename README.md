# bablfsh

"The Babel fish is small, yellow, leech-like, and probably the oddest thing in the Universe. It feeds on
brainwave energy received not from its own carrier, but from those around it. It absorbs all unconscious
mental frequencies from this brainwave energy to nourish itself with. It then excretes into the mind of its
carrier a telepathic matrix formed by combining the conscious thought frequencies with nerve signals picked
up from the speech centres of the brain which has supplied them. The practical upshot of all this is that
if you stick a Babel fish in your ear you can instantly understand anything said to you in any form of
language. The speech patterns you actually hear decode the brainwave matrix which has been fed into your
mind by your Babel fish."

Of course, since this tool is targeted specifically at Cribl, it needed fewer vowels. And so here we are...

# What's it do?

Cribl supports sending to and receiving from a variety of destinations and sources. To facilitate testing,
it's useful to have a tool that can send and receive data to and from these destinations and sources. Like
the Babel fish, bablfsh nourishes itself on the detrius of the internet and excretes made-up events into
Cribl Stream while also absorbing the unconscious mental frequencies emitted by Cribl Stream.

# How do I use it?

## Building

- TODO: Add detailed instructions for building. For now, you'll need a Rust toolchain installed, and then from the
  repo root, run `cargo build --release`. The resultant binary will be in `target/release/bablfsh`.

## Running

- Better configuration to come, but for now either run the binary or `cargo run` from repo root. A config
  file will be expected in `./config/default.json5` or `.config/local.json5` (JSON5 is a superset of JSON).
  See the included `./config/default.json5` for an example configuration.

# Components:

## Emitters

- Emitters are composed of:
  - A data generator
  - An event formatter/serializer
  - A transport
- Emitters can be configured to send events:
  - At a configurable rate in events per second
  - For a configurable number of events per cycle
  - For a configurable number of cycles before exiting (or can run forever)
  - With a configurable delay between cycles of events
  - With or without TLS support (TCP only, HTTP in the future)
  - Over TCP, UDP, or HTTP (HTTP support is not yet implemented)
  - To a configurable IP/port
  - With various event formats:
    - Syslog 3164
    - Syslog 5424 (octet-count framing coming soon)
    - ... more to come ...

## Absorbers

TODO: Not yet implemented