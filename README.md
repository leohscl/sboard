<details>
<summary>Table of Contents</summary>

- [Sboard](#Sboard)
  - [Installation](#installation)
  - [License](#license)

</details>

<!-- cargo-rdme start -->

![Demo](https://github.com/ratatui-org/ratatui/blob/1d39444e3dea6f309cf9035be2417ac711c1abc9/examples/demo2-destroy.gif?raw=true)

<div align="center">

[![Crate Badge]][Crate] [![Docs Badge]][API Docs] [![CI Badge]][CI Workflow] [![Deps.rs
Badge]][Deps.rs]<br> [![Codecov Badge]][Codecov] [![License Badge]](./LICENSE) [![Sponsors
Badge]][GitHub Sponsors]<br> [![Discord Badge]][Discord Server] [![Matrix Badge]][Matrix]
[![Forum Badge]][Forum]<br>

[Ratatui Website] · [API Docs] · [Examples] · [Changelog] · [Breaking Changes]<br>
[Contributing] · [Report a bug] · [Request a Feature] · [Create a Pull Request]

</div>

# Ratatui

Sboard (SLURM board) is a binary for displaying SLURM jobs in a convenient UI.
As of now, it is mostly a sacct wrapper with a way to view job logs

## Installation

## License

[MIT](./LICENSE)

Todo:
- [ ] Add search filters
- [ ] Job array handle mixed states
- [ ] View partition (separate tool ?)
- [x] Add reportseff
- [x] Add Older/Younger option to see different time of execution
- [x] Handle when there are no jobs after filter
- [x] Go back to logs after viewing
- [x] Handle no logs
- [x] Finished/Running crashes
- [x] remove logs in release
- [x] job array logs
