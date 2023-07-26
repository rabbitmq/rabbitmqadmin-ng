# rabbitmqadmin v2

This repository contains an early version of a `rabbitmqadmin` v2.

## Project Goals

For version 2.0, RabbitMQ Core Team has a few ideas in mind:

 * For v2, breaking changes are OK. `rabbitmqadmin` hasn't seen a revision in thirteen years
 * Python is a lovely language but we'd like at least some RabbitMQ CLI tools to be standalone binaries. For `rabbitmqadmin` specifically, there are very few reasons not to build and distribute it that way
 * v2 should be a standalone tool distributed via GitHub and not a special `rabbitmq_management` endpoint
 * We'd like to improve validation of flags and arguments, even if the interface changes somewhat in the process
 * Output should be revisited: what columns are output by default, whether columns should be selectable
 * Support for JSON and CSV was a popular addition in `rabbitmqctl`, `rabbitmq-diagnostics`, etc. Perhaps `rabbitmqadmin` should support them, too?

and on top of that, we'd like to expand the Rust expertise on our team, just like
we did with Elixir in the 2nd generation of `rabbitmqctl`, `rabbitmq-diagnostics`, `rabbitmq-upgrade`,
and so on.

## Project Maturity

This project is under heavy development, is very incomplete, and should
not be used by anyone at this time.

## License

This tool, `rabbitmqadmin` (v2 and later versions), is dual-licensed under
the Apache Software License 2.0 and the MIT license.
