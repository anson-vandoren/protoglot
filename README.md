# bablfsh

"The Babel fish is small, yellow, leech-like, and probably the oddest thing in the Universe. It feeds on
brainwave energy received not from its own carrier, but from those around it. It absorbs all unconscious
mental frequencies from this brainwave energy to nourish itself with. It then excretes into the mind of its
carrier a telepathic matrix formed by combining the conscious thought frequencies with nerve signals picked
up from the speech centres of the brain which has supplied them. The practical upshot of all this is that
if you stick a Babel fish in your ear you can instantly understand anything said to you in any form of
language. The speech patterns you actually hear decode the brainwave matrix which has been fed into your
mind by your Babel fish."

Of course, since this will send and receive from Cribl Stream, it needed fewer vowels. And so here we are...

# What's it do?

Cribl supports sending to and receiving from a variety of destinations and sources. To facilitate testing,
it's useful to have a tool that can send and receive data to and from these destinations and sources. Like
the Babel fish, bablfsh nourishes itself on the detrius of the internet and excretes made-up events into
Cribl Stream while also absorbing the unconscious mental frequencies emitted by Cribl Stream.

# Components:

## Fake data absorber

- Scours the best parts of the internet for amusing text from which to synthesize log-type messages
- Combines the log message with other fields requested by its configuration such as:
  - Timestamp
  - Hostname
  - Source
  - Sourcetype
  - Index
  - PRI
  - PID
  - Application name
  - Event ID
  - Version
  - Message
- Data absorbers can be configured for a maximum throughput specified in events per second
- Data absorbers will have a mechanism to modulate the actual throughput, up to the maximum,
  based on backpressure signals.

## Emitters

- Emitters are composed of:
  - A data generator
  - An event formatter/serializer
  - A transport
