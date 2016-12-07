# Design

**Stabping** uses a self-hosted client-server architecture. A server component
stays always running, constantly collecting network latency and speed data at
set intervals, and serving the assets for a web-based client. The client
requests data from the server and displays it them in an interactive graph. The
server can also *broadcast* live data to the client which dynamically updates
the graph.

## The Stack ##

Stabping's server component is written in [Rust](https://www.rust-lang.org/) with
the [Iron](http://ironframework.io/) web framework. We chose this stack a)
because we love Rust, and b) so that we could achieve good performance on
low-powered hardware such as the Raspberry Pi.

Stabping's client component is written in native ES6 JavaScript targeting
modern browsers with the lightweight [Preact](https://preactjs.com/) reactive
view framework and the amazing [Dygraphs](http://dygraphs.com/) time-series
charting library.

## Nomenclature and Overview ##

*The bolded and italicized terms below carry special/specific meaning that we
use in the rest of this document and in the code*

Stabping is distributed as a single binary containing the server and
compiled-in ready-to-go client assets. This binary runs with a
**configuration** loaded from a configuration file that specifies what ports
the server should listen on for browser connections.

Stabping utilizes the concept of a **target**. A **target** (or **kind** of
target) is simply some statistic of the network that can be monitored, be it
TCP ping latency, HTTP download speeds, or DNS lookup times (currently Stabping
only supports TCP Ping).

Current **target kinds** (with their specific meaning of *addrs* in
**options**, and *value* in **data**)

* TCP Ping
    * *addrs* is list of `host:port` strings, e.g. `google.com:80`
    * *value* is latency in TCP handshake expressed in microseconds

Each target has its own **options**, user-configurable settings such as how
often to collect data and which hosts to ping.

Specifically, **options** include

* *interval* (integer): milliseconds between each data collection process
  (frequency of data collection)
* *avg_across* (integer): over how many attempts should a single data point be
  an average across
* *pause* (integer): milliseconds to wait between the attempts that make up the
  final average
* *addrs* (list of strings): list of "addresses" (which have different meanings
  for each target)

One way to interpret **options** is instructing each **target** to "ping/go out
to each address in *addrs* every *interval* milliseconds *avg_across* times
with *pause* milliseconds between each attempt and return the average of those
attempts as one datapoint for that point in time"

As **options** is user-configurable, it also includes a **nonce** value, an
integer distinguishing a specific "version" of **options**. This value is
incremented every time **options** is changed.

Finally, each **target** is associated with its collected **data**, its series
of collected datapoints. In general **data** is a bunch of (*addr* <-> *time*
<-> *value*) associations where *addr* is the specific "address" in *addrs* the
datapoint was collected for, *time* is the time (expressed as seconds from
epoch) the datapoint was collected, and *value* is the actual value for that
point (which have different representations for each target)

## The Server ##

The server is responsible for:

1. Collecting data
2. Persistently storing the data
3. Pushing live data to any connected client
4. Sending back persistent data to any client that requests it
5. Serving an endpoint for retrieving and updating **options**
6. Serving the client's web assets (HTML, JavaScript, and CSS)

#### Collecting Data

The server's main thread spawns one **worker** thread for each kind of target.
Each thread holds the sending end of a MPSC (multiple-producer-single-consumer)
channel, and the main thread holds the receiving end. We spawn separate threads
for each worker as it makes the results and timings easier to reason about, and
prevents one locked up worker from blocking others.

Every *interval* milliseconds, each worker spawns subthreads (one for each
address in *addrs*) which do the actual data collection (e.g. measuring latency
of a TCP handshake). These additional threads are spawned so that one address
blocking does not prevent others from returning. At the end of *interval*
milliseconds, the worker thread then combines all of these individual
collections into a `TargetResults` package, and sends it back to the main
thread. This is an array of 32-bit integers [kind, nonce, time, value1, value2,
...], where the values are ordered in the order of the addresses as they appear
in *addrs*.

#### Persistently Storing the Data

The server manages three separate files for each **target**: an options file,
an index file, and a data file.

The options file is simply a JSON dump of the current **options** of the
**target**.

The index file is a per-target global mapping of numerical identifiers (called
*indices*) to unique addresses that appear (or have appeared before) in
*addrs*.

The data file is a large binary file of all the raw data for this target,
stored as back-to-back triplets of 32-bit integers representing [*time*,
*index*, *value*]. We chose this storage format as it allows for easy and
time-efficient binary searching of specific times, does not need to rewritten
with the addition/removal of new addresses, and is space-efficient.

As the main thread receives data from the **workers**, it appends it to the
data file (while converting between the formats).

#### Pushing Live Data to the Client

The main thread then *broadcasts* the data to all connected clients via
websockets in the same format it received from the **workers**, an array of
32-bit integers [kind, nonce, time, value1, value2, ...].

#### Sending Back Persistent Data

Endpoint: `POST /api/target/<kind>`.

Upon receiving a request specifying a lower and upper time bound at this
endpoint, the server `mmap`'s the requested **target**'s data file, and binary
searches for the start and end points in the file. Then it writes out (to an
HTTP response) a back-to-back series of arrays of 32-bit integers [time,
value1, value2, ...], with the values in the order of the addresses as they
appear in *addrs*. This entails figuring out which *indices* are those of
current addresses in *addrs* and ordering them correctly. We chose this network
transfer format as it is extremely space-efficient, allowing for rapid transfer
of large amounts of data over the network.

#### Serving **Options**

Endpoint: `GET/PUT /api/target/<kind>`.

This is straightforward JSON retrieve and update endpoint, with the addition
that on `PUT`s to update the **options**, the server sends back the new
(incremented) nonce (and writes the update to the **target**'s options file).

#### Serving Web Assets

Stabping aims to be minimal (and really zero, if defaults are used)
configuration. This includes our relatively unique choice to bundle web assets
(HTML, JavaScript, and CSS) with our server binary, meaning users can run one
binary and have everything work without worrying about placing web assets in
the correct place with correct permissions where the web server can discover.

We accomplish this compiling-in of web assets through a custom build procedure
hooked into Rust's build pipeline in `build.rs`. The routine in `build.rs`
essentially discovers the necessary web assets at compile time and *inlines*
them into the server's Rust source code. In our web server code we then have
logic that interacts with these inlined assets, properly serving them at
`/assets` while setting things like `Content-Type` correctly.

## The Client ##

The client is responsible for:

1. Receiving live data
2. Fetching persistent data
3. Allowing the user update **options**

in addition to providing many features for interacting and viewing the data.

#### Receiving Live Data

Stabping's JavaScript web client takes advantage of ES6's [Typed Arrays and
Buffers](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Typed_arrays),
allowing us to interact with the binary network transfer format the server
uses.

To receive live data, the client establishes a websocket connection with the
server, and then relays the incoming data to the corresponding `Target`
Components based on the **target kind**. Each `Target` Component is then
responsible for appending the data to the graph.

#### Fetching Persistent Data

Each `Target` Component includes a *preset* selector that specifies how far
back the view of the graph should span. On load and when this *preset* changes
the client `POST`s to `/api/target/<kind>` for data as necessary to fetch data
that it doesn't already have. Each `Target` Component keeps track of how much
data it already has via `state.leftLimit` which is the lower time bound of
the data it has. In-browser, the data is stored directly in the format the
Dygraphs understands, an large array of [time, value1, value2, ...] arrays
representing each datapoint.

#### Updating **Options**

Each `Target` Component can mount an `Options` Component that loads a UI
allowing the user to change **options**. On 'Save', the `Target` Component then
`PUT`s JSON to `/api/target/<kind>` which does the actual updating on the
server, and returns the new *nonce*.
