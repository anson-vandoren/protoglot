# protoglot

# What's it do?

To facilitate various testing of event pipelining (etc.) tools, it's useful to have a tool that can send and
receive data to and from various destinations and sources.

# How do I use it?

## Building

1. Install the Rust toolchain: https://www.rust-lang.org/tools/install
2. Clone this repo, then `cargo build --release` from the repo root.
3. Copy the resultant binary from `target/release/protoglot` to a location in your path.

## Running from Docker

If you don't want to or need to build the binary yourself, you can run it in a Docker container. The Docker image is available on
Docker Hub at `ansonvandoren/protoglot`. To run it, you can use the following command:

```bash
docker run --rm ansonvandoren/protoglot --help
```

When running protoglot in a Docker container, the configuration options and command line arguments below still apply, but
you would need to mount a volume to the container to provide a config file. In this case it's easier to just use the 
command line arguments to configure the tool.

**Examples**:

```bash
$ docker run --rm ansonvandoren/protoglot --host 172.17.0.1 # or host.docker.internal for macfolk
```

## Running

- Run `protoglot --help` to see the available options.
- Run `protoglot config` to generate a default config at `~/.config/default.json5`.
- If you have a JSON5 config file located in your system's config directories, that will be used in place of the default.
  - On Linux, the path is `~/.config/protoglot/config.json5`
  - On Windows, the path is `C:\Users\<username>\AppData\Roaming\ansonvandoren\protoglot\config\config.json5`
  - On macOS, the path is `~/Library/Application Support/com.ansonvandoren.protoglot/config.json5`
- If you pass a `--file` argument, protoglot will read the config from that file instead of the default or system config file.
- Only one of the above config files will be used (in the order described), and whichever file is selected must be contain all fields present in the default config file.
- If you wish to override specific fields from the command line, you can do so with the appropriate flags as described in the help output. Command line arguments will override both config files and environment variables.


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
