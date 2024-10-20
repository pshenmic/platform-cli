# CHANGELOG

# v0.1.5
Verbose logging and fix for mainnet network type (was only accepting `dash` as a value for mainnet)

## What's Changed
* Add more verbose logging to the command by @pshenmic in https://github.com/pshenmic/platform-cli/pull/10
* Allow mainnet as a network type by @pshenmic in https://github.com/pshenmic/platform-cli/pull/11


**Full Changelog**: https://github.com/pshenmic/platform-cli/compare/v0.1.4...v0.1.5

# v0.1.4
Accept different private key encoding types in the input file

## What's Changed
* Accept all possible private key encoding types (base64, hex, wif) by @pshenmic in https://github.com/pshenmic/platform-cli/pull/8
* Hardcode DPNS data contract in the code by @pshenmic in https://github.com/pshenmic/platform-cli/pull/9

## Breaking Changes
* Introduced new required `--network` flag

**Full Changelog**: https://github.com/pshenmic/platform-cli/compare/v0.1.3...v0.1.4

# v0.1.3
Fix GLIBC issue for older systems (now being build under Ubuntu 20.04), and fix for whitespaces or newlines in the input file data

## What's Changed
* Downgrade Linux GLIBC requirement by @pshenmic in https://github.com/pshenmic/platform-cli/pull/6
* Strip input file whitespaces and newlines by @pshenmic in https://github.com/pshenmic/platform-cli/pull/7


**Full Changelog**: https://github.com/pshenmic/platform-cli/compare/v0.1.2...v0.1.3

# v0.1.2
Linux ARM64 builds support

## What's Changed
* Added Linux arm64 builds on CI by @pshenmic in https://github.com/pshenmic/platform-cli/pull/4
* Removed rs-drive from the project dependencies

**Full Changelog**: https://github.com/pshenmic/platform-cli/compare/v0.1.1...v0.1.2

# v0.1.1
Minor documentation updates and cleanup

## What's Changed
* Updated application and README.md documentation by @pshenmic in https://github.com/pshenmic/platform-cli/pull/5

**Full Changelog**: https://github.com/pshenmic/platform-cli/compare/v0.1.0...v0.1.1

# v0.1.0

First release, a basic CLI application structure (3 commands implemented) and cross-compile multi arch CI release builds.

A first, MVP version, allows you to create and broadcast these actions in the Dash Platform network:
* Identity Credit Withdrawal
* Register DPNS Name
* Masternode vote on contested DPNS name

## What's Changed
* Basic app structure
* Implemented Identity Credit Withdrawal command
* Implemented Register DPNS Name command
* Implemented Identity Credit Withdrawal command
* Documentation & Changelog
* Added GitHub actions by @pshenmic in https://github.com/pshenmic/platform-cli/pull/1
* Cross-compiled CI builds on CI by @pshenmic in https://github.com/pshenmic/platform-cli/pull/3

## New Contributors
* @pshenmic made their first contribution in https://github.com/pshenmic/platform-cli/pull/1

**Full Changelog**: https://github.com/pshenmic/platform-cli/commits/v0.1.0