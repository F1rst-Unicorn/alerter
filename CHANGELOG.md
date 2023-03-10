# Change Log

All notable changes to this project will be documented in this file.

## Unreleased

### Maintenance

* Update library dependencies

## [2.0.4]

### Fixed

* Fix SAS verification. This was broken for unknown reasons.

### Maintenance

* Update Rust Edition 2021

* Update library dependencies

## [2.0.3]

* Fix linter issues

* Update library dependencies

## [2.0.2]

### Maintenance

* Increase timeout for server replies to lower system load

* Update dependencies

* Fix linter issues

## [2.0.1]

### Fixed

* `alert` client now only requires the `socket_path` in the config file again.

### Added

* Documented SAS client verification.

## [2.0.0]

### Added

* Support for the [Matrix Protocol](https://matrix.org/). This is a breaking
  change for the configuration file `alerter.yml`.

## [1.0.8]

### Fixed

* Fix termination during startup if no socket is present from a previous run

## [1.0.7]

### Maintenance

* Add system tests

* Configure logging via file

* Move config files into own `/etc` subdirectory

* Update dependencies

## [1.0.6]

### Maintenance

* Fix linter issues

## [1.0.5]

### Maintenance

* Update dependencies

## [1.0.4]

### Fixed

* Fix crash on message transmission

## [1.0.3]

### Added

* Version information now contains the commit hash and date of the build

* Inform systemd when service is ready

### Maintenance

* Update dependencies

## [1.0.2]

### Maintenance

* Update dependencies

## [1.0.1]

### Fixed

* Don't truncate custom field values containing `:`

## [1.0.0]

### Added

* System integration ([#2](https://j.njsm.de/git/veenj/alerter/issues/2))

* Reliable message transmission
  ([#1](https://j.njsm.de/git/veenj/alerter/issues/1))

* Documentation ([#3](https://j.njsm.de/git/veenj/alerter/issues/3))
