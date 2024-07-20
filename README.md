# protoglot

# What's it do?

Cribl supports sending to and receiving from a variety of destinations and sources. To facilitate testing,
it's useful to have a tool that can send and receive data to and from these destinations and sources.

# How do I use it?

## Building

1. Install the Rust toolchain: https://www.rust-lang.org/tools/install
2. Clone this repo, then `cargo build --release` from the repo root.
3. Copy the resultant binary from `target/release/protoglot` to a location in your path.

## Running from Docker

If you don't want to or need to build the binary yourself, you can run it in a Docker container. The Docker image is available on Docker Hub at `ansonvandoren/protoglot`. To run it, you can use the following command:

```bash
docker run --rm ansonvandoren/protoglot --help
```

When running protoglot in a Docker container, the configuration options and command line arguments below still apply, but you would need to mount a volume to the container to provide a config file. In this case it's easier to just use environment variables or command line arguments to configure the tool.

**Examples**:

```bash
$ docker run --rm ansonvandoren/protoglot --host 172.17.0.1 # or host.docker.internal for macfolk
```

## Running

1. Run `protoglot --help` to see the available options.
2. With no options, `protoglot` will use the config found in the repo under `./config/default.json5` (which is bundled inside the binary itself at compile time):

```json5
{
  emitter: {
    host: "localhost", // FQDN or IP address
    port: 9514,
    tls: false, // defaults to false if omitted
    protocol: "tcp", // "tcp" or "udp"
    rate: 1000, // in events per second
    // currently supported message types:
    // - "syslog3164": RFC 3164 syslog message
    // - "syslog5424": RFC 5424 syslog message
    messageType: "syslog3164",
    numEmitters: 1, // number of concurrent emitters to run, each at the EPS rate above
    eventsPerCycle: 10000, // number of events to send in each cycle
    numCycles: 1, // number of cycles to send, use 0 for infinite
    cycleDelay: 10000, // delay in milliseconds between cycles
  },
  absorber: {
    updateInterval: 5000, // in milliseconds
    messageType: "syslog3164",
    listenAddresses: [], // absorber does not listen by default
  }
}

```

3. If you have a JSON5 config file located in your system's config directories, that will be used in place of the default.
  a. On Linux, the path is `~/.config/protoglot/config.json5`
  b. On Windows, the path is `C:\Users\<username>\AppData\Roaming\ansonvandoren\protoglot\config\config.json5`
  c. On macOS, the path is `~/Library/Application Support/com.ansonvandoren.protoglot/config.json5`

4. If you pass a `--file` argument, protoglot will read the config from that file instead of the default or system config file.
5. Only one of the above config files will be used (in the order described), and whichever file is selected must be contain all fields present in the default config file.
6. If you wish to override specific fields from the command line, you can do so with the appropriate flags as described in the help output. Command line arguments will override both config files and environment variables.


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

- Absorbers are composed of:
  - A listening address, port, and protocol
  - A message checker (to verify correctness to some degree)