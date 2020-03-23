# gfx-extras
[![Matrix](https://img.shields.io/badge/Matrix-%23gfx%3Amatrix.org-blueviolet.svg)](https://matrix.to/#/#gfx:matrix.org)
[![Build Status](https://travis-ci.org/gfx-rs/gfx-extras.svg?branch=master)](https://travis-ci.org/gfx-rs/gfx-extras)

Extra libraries to help working with gfx-hal:
  - [![Crates.io](https://img.shields.io/crates/v/gfx-descriptor.svg)](https://crates.io/crates/gfx-descriptor) - descriptor allocator
  - [![Crates.io](https://img.shields.io/crates/v/gfx-memory.svg)](https://crates.io/crates/gfx-memory) - memory allocator

## Contributing

We are using Github with PR-based workflow into the `master` branch for the cutting edge development. The released versions are branched out, and some patches can be back-ported into the release branches for patch updates. Only linear history is allowed in all the branches, no merge commits. Every commit has to be green on CI in order to easy bisection later on.

We are using `rustfmt` stable with default configuration, not enforced by CI.
