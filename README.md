# Stabping

[![Travis Build Status](https://travis-ci.org/icasdri/stabping.svg?branch=master)](https://travis-ci.org/icasdri/stabping)
[![AppVeyor Build status](https://ci.appveyor.com/api/projects/status/5qi6atyjt69nsmkx/branch/master?svg=true)](https://ci.appveyor.com/project/icasdri/stabping)

**Stabping** is a lightweight and simple-to-use network connection monitor. It
continuously tests latency (speed and other metrics coming soon) in a
user-configurable fashion and allows you to view the data in live interactive
charts.

The program itself is designed to be run on an always-on computer so that it
can constantly be collecting data. Using a web browser, you can then interact
with the progam and the collected data.

## Installation

**Stabping** does not require any installation: it is distributed for mutliple
platforms as a self-contained portable executable. To get started, simply
download the appropriate pre-built binary for your platform from the
*Downloads* section in
[Releases](https://github.com/icasdri/stabping/releases). If you would prefer
to build **Stabping** yourself, see [Manual Build](#manual-build) below.

**Supported platforms**

|Platform            |Platform Code                |
|--------------------|-----------------------------|
|Linux (Raspberry Pi)|`arm-unknown-linux-gnueabihf`|
|Linux (64-bit)      |`x86_64-unknown-linux-gnu`   |
|MacOS (64-bit)      |`x86_64-apple-darwin`        |
|Windows (64-bit)    |`x86_64-pc-windows-gnu`      |

After downloading the appropriate *zip* file from the *Downloads* section in
[Releases](https://github.com/icasdri/stabping/releases), extract it to find
the following files:

* `stabping` (or `stabping.exe` if you're on Windows)
    * This is the actual executable binary
* `stabping_config.json`
    * This is a sample configuration file with default values

## Usage

#### Getting Started

To run **Stabping**, simply put the configuration file in one of the following
places

* the current working directory (only relevant when you're running from
  Terminal or Command Prompt)
* the directory where `stabping` (or `stabping.exe` if you're on Windows) is
  located
* your user directory (i.e. "home" directory, e.g. `~/` or `C:\Users\yourname`)
* a global configuration directory (e.g. `/etc`)

and then run `stabping` or `stabping.exe`!

Once **Stabping** is running, you can go to `http://address:web_port` in a web
browser to interact with it, where `address` is the IP address or DNS name of
the computer you're running `stabping` or `stabping.exe` on (if you ran it on
your local computer, this would be `localhost`), and `web_port` is the
web-listening port specified in `stabping_config.json` (by default `5001`).

**tl;dr** extract the zip, run `stabping` or `stabping.exe`, and go to
`http://localhost:5001` in a web browser (assuming you're running on your local
computer with the default configuration).

#### Using the Web Interface

The web interface displays a live interactive graph for each network metric
(currently only *TCP Ping*, aka. TCP connection latency). By default, this
graph displays the past hour's worth of data, but this can be adjusted to any
time interval using the *Base Time Interval* drop down. The graph will
live-update with new data as they are being colleted. (if you just installed
**Stabping**, give it a few minutes to collect some data -- you can watch as
the live data rolls in!)

The graph is *interactive*!

* Hover over it to inspect the values of individual data points.
* Click and drag left-to-right to zoom into a smaller time window.
* Click and drag up-and-down to zoom into a smaller value range.
* Double anywhere blank to unzoom back out to *Base Time Interval* worth of
  data.

The graph will automatically adjust the vertical axes to best accomodate the
vertical range of data (when zooming and when live data arrives). To prevent
this and *pin/lock* the vertical value range to what is currently visible,
check the *Pin/Lock value range* checkbox (and make sure you then unzoom the
graph).  Uncheck this box to return to automatic adjustment.

The graph can also dynamically calculate and display a [moving/rolling
average](https://en.wikipedia.org/wiki/Moving_average) to reduce the
"spikyness" of the data and more easily see general trends. Simply adjust *Roll
avg over __ point(s)* to the number of points you want to roll over.

You can configure how (e.g. which servers to ping) and how often **Stabping**
collects data for a specific metric by clicking on the corresponding *gear
icon*. This will display a configuration interface in which you can make
changes. Once you're satisfied, click *Save* -- **Stabping** will adjust its
data collection processes accordingly and the graph will update as needed.

## Manual Build

**Stabping** is written in [Rust](https://www.rust-lang.org/) and requires a
working (preferably recent) Rust toolchain to build. Specifically, **Stabping**
uses [`cargo`](http://doc.crates.io/guide.html), Rust's package manager to
build.  Additionally, the build process pulls several dependencies from
[npm](https://www.npmjs.com/) for the web client, and therefore also requires a
working (preferably recent) [Node](https://nodejs.org/en/) toolchain.

After ensuring that `rustc`, `cargo`, and `npm` (or `npm.cmd` on Windows) are
available in your `PATH`, and after cloning this repo, run any of the
following.

To build a "debug" version

    cargo build

To build a "release" version

    cargo build --release

To build (a "debug" version) and run it directly

    cargo run

## License and Acknowledgements

**Stabping** builds on top of the work of a number of amazing open-source
libraries, frameworks, and assets.

On the server (always running program) side:

* [Iron](http://ironframework.io/): a web framework
* [WS-RS](https://ws-rs.org/): a lightweight event-driven WebSockets library
* [memmap](https://github.com/danburkert/memmap-rs): a cross-platform API for
  memory-mapped I/O
* [rustc-serialize](https://github.com/rust-lang-nursery/rustc-serialize): a
  compiler-assisted serialization library
* [chrono](https://lifthrasiir.github.io/rust-chrono/): a date/time library

On the client (web interface) side:

* [Dygraphs](http://dygraphs.com/): a powerful JavaScript time-series charting
  library
* [Preact](https://preactjs.com/): a fast reactive components and virtual DOM
  based JavaScript framework
* [Fira Fonts](https://mozilla.github.io/Fira/): a new and modern typeface

**Stabping** as a whole is `Copyright 2016 icasdri` and licensed under the GNU
GPLv3+, see [COPYING](https://github.com/icasdri/stabping/blob/master/COPYING/)
for details.
