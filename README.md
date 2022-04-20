# twentyfaces
The Fiend with Twenty Faces on Twitter.

## What's this?

Automatically switch your Twitter profile (user name, icon, and description) depend on your tweet.
(https://github.com/karno/Masquerade with Rust.)

### What's difference between twentyfaces and masquerade?
Masquerade works only on UserStreams (already EoL'ed). twentyfaces works well with REST (polling-base) API.

## Prerequisites

- [cargo](https://www.rust-lang.org/tools/install)
- [twitter OAuth 1.1 token](https://developer.twitter.com/ja/docs/basics/authentication/guides/access-tokens)

## Installation

1. clone this repository
2. build with cargo: `cargo build --release`
3. took out `./target/release/twentyface(.exe)` to your favorite location
4. execute `twentyfaces` and you'll navigate to initial setup.

## License

MIT
