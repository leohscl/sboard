<details>
<summary>Table of Contents</summary>

- [Sboard](#Sboard)
  - [Installation](#installation)
  - [License](#license)

</details>

<!-- cargo-rdme start -->

![Demo](https://github.com/leohscl/sboard/blob/22fde5c552ac5075d3980d5a1ac9b8642512b854/assets/sboard_render.gif)

# Sboard

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
