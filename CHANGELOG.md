# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- gzip decompression based on `Content-Encoding` header for the HTTP absorber

## [0.3.0](https://github.com/anson-vandoren/protoglot/compare/v0.2.1..v0.3.0) - 2025-04-25

### Added

- A HTTP absorber. To use, run like `protoglot absorber http://127.0.0.1:12345`, for example.
  This example will listen for HTTP/1.1 on the specified IP address and port, with TLS disabled.

- Various flags for HTTP2 and TLS for the new HTTP absorber.
  - `--http2`: listens for HTTP2 instead. Note that this _also_ enables TLS.
  - `--https`: listens for HTTPS only, on HTTP/1.1. Unencrypted HTTP connections will be rejected.
  - `--self-signed`: when TLS is enabled from one of the two options above, serves TLS using
    a bare self-signed cert, which will be printed to stdout on startup, and also saved to
    `/tmp/protoglot/` dir.
  - `--private-ca`: when TLS is enabled from one of the two options above, serves TLS using
    a server cert that is signed by a private CA cert. Both the server cert and the CA cert will
    be printed to stdout on startup, and also saved to `/tmp/protoglot/` dir.

## [0.2.1](https://github.com/anson-vandoren/protoglot/compare/v0.2.0..v0.2.1) - 2025-04-20

### Added

- This changelog.

- `protoglot config` sub-command to write out default config to default config path. 
  Use the `--overwrite true` flag to overwrite an existing config file at that path.


[//]: # (Change types: Added, Changed, Deprecated, Removed, Fixed, Security)
