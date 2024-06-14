# Hedgehog

Hedgehog is a Rust library designed to make it easy to interact with the [Posthog API](https://posthog.com/docs/api/overview), a powerful open-source product analytics platform.

## Notice

This repository contains software that was originally developed for internal use by our organization. We have chosen to open-source this software in the hopes that it may be of use to others.

Please note the following important points:
- While we are making this software available to the public, we will not be providing external support. If you choose to use this software, please understand that you do so entirely at your own risk. The source code is available for you to use and modify as you wish, within the bounds of the included license.

## Why Hedgehog?

There already exists a few Rust libraries for interacting with the Posthog API, but we found them all to be lacking in some way: 
- We wanted a library that was easy to use, and that provided a simple interface for interacting with the API.
- The official library is a very basic SDK that only has support for capturing events, and is not actively maintained.
- Other libraries we found were either incomplete, or lacked an async interface.

## Supported Posthog Features

- [x] Identify users
- [x] Capture events
- [x] Capture events in batch
- [x] Record page views
- [x] Record screen views
- [x] Evaluate feature flags
- [x] Include feature flag information when capturing events
- [x] Feature flag called event
- [x] Override GeoIP information when capturing events based on IP address
- [x] Early access features retrieval
- [x] Early access feature enrollment
