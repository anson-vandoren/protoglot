# protoglot

Protoglot is a fast, practical event generator and receiver for testing data pipelines.

It can emit realistic-ish event streams over TCP, UDP, HTTP, HTTPS, and TCPS, and it can also run as an absorber that listens for incoming traffic and reports live throughput stats. It is built for quick local testing, pipeline source validation, and load-ish workflows where you need a simple binary that speaks the protocols your pipeline expects.

## Why Use It?

- Test receivers without wiring up a full upstream system.
- Exercise TCP, UDP, HTTP, HTTPS, and TLS paths from one CLI.
- Generate Syslog, NDJSON, and Splunk HEC payloads.
- Batch Splunk HEC events into normal multi-event HTTP POST bodies.
- Run an absorber to validate incoming events and watch event/byte rates.
- Keep repeated workflows simple with built-in profiles and config templates.

## Quick Start

Build it:

```bash
cargo build --release
```

Run a local Splunk HEC emitter profile:

```bash
protoglot --profile splunk-hec
```

That sends 100 Splunk HEC events to `http://127.0.0.1:8088/services/collector/event` using the default token `protoglot-hec-token`.

Override only what you need:

```bash
protoglot --profile splunk-hec --host 127.0.0.1 --hec-token dev-token --events 10000
```

Start an absorber:

```bash
protoglot absorber --listen tcp://127.0.0.1:9514 --message-type syslog5424
```

## Built-In Profiles

Profiles are complete runnable presets. Use them when you want the common protocol/message defaults without remembering every flag.

```bash
protoglot --profile splunk-hec
protoglot --profile tcp-syslog3164
protoglot --profile tcp-syslog5424
protoglot --profile udp-syslog3164
protoglot --profile http-ndjson
```

Current profiles:

| Profile | Protocol | Host | Port | Message type | Events |
| --- | --- | --- | --- | --- | --- |
| `splunk-hec` | HTTP | `127.0.0.1` | `8088` | Splunk HEC | `100` |
| `tcp-syslog3164` | TCP | `127.0.0.1` | `9514` | Syslog 3164 | `100` |
| `tcp-syslog5424` | TCP | `127.0.0.1` | `9514` | Syslog 5424 | `100` |
| `udp-syslog3164` | UDP | `127.0.0.1` | `9514` | Syslog 3164 | `100` |
| `http-ndjson` | HTTP | `127.0.0.1` | `8080` | NDJSON | `100` |

Profiles are just a base layer. CLI flags still override them:

```bash
protoglot --profile tcp-syslog5424 --host 10.0.0.12 --rate 5000 --events 100000
```

## Splunk HEC

Protoglot can emit Splunk HEC-formatted HTTP payloads for testing HEC-compatible receivers and pipeline sources.

```bash
protoglot --profile splunk-hec
```

The HEC profile sends newline-delimited HEC event envelopes in each HTTP POST body. This avoids the wasteful one-event-per-request path while keeping the payload stream easy to parse and inspect.

Default HEC settings:

| Setting | Value |
| --- | --- |
| URL | `http://127.0.0.1:8088/services/collector/event` |
| Auth header | `Authorization: Splunk protoglot-hec-token` |
| Batch size | `100` events per POST |
| Total events | `100` |

Useful overrides:

```bash
protoglot --profile splunk-hec --hec-token xenomux-dev --events 50000
protoglot --profile splunk-hec --host 10.0.0.12 --port 8088
protoglot --profile splunk-hec --hec-batch-size 500 --rate 10000
```

HEC events include varied envelope metadata and event shapes, including object events, string events, metric-like events, audit-like events, and nested JSON. That variety is intentional: it is meant to exercise source parsing behavior, not just prove that a single happy-path JSON shape works.

## Emitters

With no subcommand, Protoglot runs as an emitter.

```bash
protoglot [OPTIONS]
```

Common emitter options:

| Option | Meaning |
| --- | --- |
| `--profile <name>` | Start from a built-in profile. |
| `--host <host>` | Target host. |
| `--port <port>` | Target port. |
| `--protocol <protocol>` | `tcp`, `tcps`, `udp`, `http`, or `https`. |
| `--client-cert <path>` | PEM client certificate chain for TLS authentication; requires `--client-key`. |
| `--client-key <path>` | PEM private key matching `--client-cert`. |
| `--ca-cert <path>` | Additional PEM CA certificates to trust for TLS servers. |
| `--message-type <type>` | `syslog3164`, `syslog5424`, `syslog5424-octet`, `nd-json`, or `splunk-hec`. |
| `--rate <n>` | Target event rate in events per second. |
| `--events <n>` | Events per cycle. |
| `--cycles <n>` | Number of cycles. Use `0` to run forever. |
| `--cycle-delay <ms>` | Delay between cycles in milliseconds. |
| `--emitters <n>` | Number of emitter tasks to run in parallel. |
| `--hec-token <token>` | Splunk HEC token for `splunk-hec` payloads. |
| `--hec-batch-size <n>` | HEC events per HTTP POST body. |

Examples:

```bash
protoglot --profile tcp-syslog3164 --host 127.0.0.1 --events 10000
protoglot --protocol udp --host 127.0.0.1 --port 9514 --message-type syslog3164
protoglot --protocol http --host 127.0.0.1 --port 8080 --message-type nd-json
protoglot --protocol tcps --host logs.example.test --port 6514 --message-type syslog5424
protoglot --protocol tcps --host logs.example.test --port 6514 --message-type syslog5424 \
  --client-cert ./client-cert.pem --client-key ./client-key.pem
protoglot --protocol https --host ingest.example.test --port 443 --message-type nd-json \
  --client-cert ./client-cert.pem --client-key ./client-key.pem --ca-cert ./private-ca.pem
```

Client certificates are only used by TLS-enabled TCP and HTTPS emitters. `--client-cert` and `--client-key` must be supplied together and do not enable TLS implicitly. Native system roots remain trusted; use `--ca-cert` only when the server requires an additional private CA. Config files use the corresponding `clientCert`, `clientKey`, and `caCert` emitter keys.

## Test Certificates

Protoglot can create a persistent test CA and use it to sign server and client certificates. On Linux, certificates are stored in `~/.config/protoglot/certs/`; other platforms use their native Protoglot configuration directory.

Generate certificates with explicit subject alternative names:

```bash
protoglot cert generate --type server --name api --dns localhost --ip 127.0.0.1
protoglot cert generate --type client --name sender --uri spiffe://example.test/sender --email sender@example.test
```

`--ip`, `--dns`, `--uri`, and `--email` may each be repeated. The reference supplied with `--name` only controls the filenames: `<name>.pem` contains the certificate and `<name>_key.pem` contains its private key. It is not added to the certificate subject or SANs.

The first generation creates the signing CA at `certs/ca/ca.pem` with its key at `certs/ca/ca_key.pem`. Later certificates reuse that CA. If a named certificate or key already exists, Protoglot asks before replacing both; use `--force` to replace them without prompting.

Print a certificate PEM to stdout:

```bash
protoglot cert print api
protoglot cert print ca
```

For example, use a generated client certificate with an emitter:

```bash
protoglot --protocol tcps --host logs.example.test --port 6514 \
  --client-cert ~/.config/protoglot/certs/sender.pem \
  --client-key ~/.config/protoglot/certs/sender_key.pem \
  --ca-cert ~/.config/protoglot/certs/ca/ca.pem
```

## Absorbers

The absorber listens for events, validates the selected message shape, and prints live stats.

```bash
protoglot absorber --listen tcp://127.0.0.1:9514 --message-type syslog3164
```

The `--listen` value uses this format:

```text
protocol://host:port
```

Examples:

```bash
protoglot absorber --listen tcp://127.0.0.1:9514 --message-type syslog5424
protoglot absorber --listen udp://127.0.0.1:9514 --message-type syslog3164
protoglot absorber --listen http://127.0.0.1:8080 --message-type nd-json
protoglot absorber --listen http://127.0.0.1:8088 --message-type splunk-hec
```

Multiple listeners can be specified:

```bash
protoglot absorber \
  --listen tcp://127.0.0.1:9514 \
  --listen udp://127.0.0.1:9514 \
  --message-type syslog3164
```

Interactive absorber controls:

| Input | Effect |
| --- | --- |
| `rs` | Reset stats. |
| `q` | Quit. |

HTTP absorber notes:

- `--https` enables HTTPS/1.1.
- `--http2` enables HTTP/2 and implies TLS.
- `--self-signed` uses a generated self-signed cert.
- `--private-ca` uses a generated private CA and server cert.
- `--mtls` requires client certs signed by the generated private CA.
- `--auth basic` and `--auth token` enable simple auth checks for HTTP absorber testing.

## Config Files

Everything available from the CLI can also be represented in config.

Write the default user config:

```bash
protoglot config
```

By default, Protoglot writes and reads the user config at the platform config path:

| Platform | Path |
| --- | --- |
| Linux | `~/.config/protoglot/config.json5` |
| macOS | `~/Library/Application Support/com.ansonvandoren.protoglot/config.json5` |
| Windows | `C:\Users\<username>\AppData\Roaming\ansonvandoren\protoglot\config\config.json5` |

Write a profile template into the current working directory:

```bash
protoglot config --template --profile splunk-hec
```

That creates:

```text
protoglot.splunk-hec.json
```

Choose your own output path:

```bash
protoglot config --template --profile splunk-hec --output ./hec-local.json
```

Run from a config file:

```bash
protoglot --file ./hec-local.json
```

Config precedence is:

1. Built-in defaults.
2. User config file, if present.
3. Selected `--profile`, if present.
4. Explicit `--file`, if present.
5. Direct CLI flags.

This means you can keep a long-lived config file and still use CLI overrides for the things you change often:

```bash
protoglot --file ./hec-local.json --hec-token temporary-token --events 250000
```

## Development

This repo includes `mise` tasks for the common development loop.

List tasks:

```bash
mise tasks
```

Build and test:

```bash
mise run build
mise run test
mise run clippy
```

CI-style check:

```bash
mise run ci
```

Release the current `Cargo.toml`/`CHANGELOG.md` version:

```bash
mise run release
```

Prepare a new version interactively, then release it:

```bash
./release.sh --make-release
```

Without `mise`, the equivalent core commands are:

```bash
cargo build
cargo nextest run
cargo clippy --all-targets --all-features
cargo test --verbose
```

## Logging

Use `-v`, `-vv`, or `-vvv` to increase Protoglot logging verbosity.

You can also set `RUST_LOG` for more targeted Rust logging behavior:

```bash
RUST_LOG=debug protoglot --profile splunk-hec
```

## Current Message Types

| Message type | CLI value | Notes |
| --- | --- | --- |
| Syslog 3164 | `syslog3164` | Traditional BSD-style syslog-ish payloads. |
| Syslog 5424 | `syslog5424` | RFC 5424-style syslog payloads. |
| Syslog 5424 octet-counted | `syslog5424-octet` | RFC 5424 payloads with octet-count framing. |
| NDJSON | `nd-json` | Newline-delimited JSON events. |
| Splunk HEC | `splunk-hec` | Newline-delimited HEC event envelopes over HTTP/HTTPS. |

## Project Status

Protoglot is intentionally pragmatic: it is not a full load-testing suite and it is not trying to perfectly emulate every producer. It is a focused tool for generating and absorbing enough realistic data to shake out source configuration, parsing behavior, transport issues, TLS/auth paths, and throughput bottlenecks.
