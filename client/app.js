const SENTINEL_ERROR = -2100000000;
const SENTINEL_NODATA = -2000000000;

function dateAxisFormatter(epochSeconds, gran, opts) {
    return Dygraph.dateAxisLabelFormatter(new Date(epochSeconds * 1000), gran, opts);
}

function dateFormatter(epochSeconds) {
    return Dygraph.dateString_(epochSeconds * 1000);
}

function TargetGraph(divId, valFormatter) {
    var gvFormatter = function(val, opts, seriesName) {
        if (seriesName == "Time") {
            return dateFormatter(val);
        } else if (val == SENTINEL_ERROR) {
            return "Error/timeout";
        } else if (val == SENTINEL_NODATA) {
            return "No Data";
        } else {
            return valFormatter(val);
        }
    };

    this.valRange = [0, null];

    this.graph = new Dygraph(
        document.getElementById(divId),
        [[0]],
        {
            valueFormatter: gvFormatter,
            valueRange: this.valRange,
            axes: {
                x: {
                    axisLabelFormatter: dateAxisFormatter
                },
                y: {
                    axisLabelFormatter: gvFormatter
                }
            },
            animatedZooms: true,
            isZoomedIgnoreProgrammaticZoom: true,
            zoomCallback: function (lowerDate, upperDate, yRanges) {
                if (!this.graph.isZoomed()) {
                    this._updateDrawnGraph();
                }
            }.bind(this)
        }
    );

    this.data = null;
    this.labels = ["Time"];
}

TargetGraph.prototype._updateDrawnGraph = function() {
    this.graph.updateOptions({
        isZoomedIgnoreProgrammaticZoom: true,
        valueRange: this.valRange,
        file: this.data
    });
}

TargetGraph.prototype.update = function(buf) {
    if (!this.data) {
        this.data = [];
    }

    var new_data = new Int32Array(buf);
    this.data.push(new_data);

    var curMax = this.valRange[1];
    for (var i = 1; i < new_data.length; i++) {
        if (new_data[i] > curMax) {
            curMax = new_data[i];
        }
    }
    this.valRange[1] = curMax;

    if (!this.graph.isZoomed()) {
        this._updateDrawnGraph();
    }
}

TargetGraph.prototype.setSeriesLabels = function(labels) {
    this.labels.length = 1;
    this.labels.push.apply(this.labels, labels);
    this.graph.updateOptions({
        labels: this.labels
    });
}


function SPSocket(addr, cb, interval) {
    if (!interval) {
        interval = 20000;
    }

    this.socket = this.newSocket(addr, cb);

    setInterval(function() {
        if (this.socket.readyState > 1) {
            console.log("Reconnecting WebSocket...");
            this.socket = this.newSocket(addr, cb);
        }
    }.bind(this), interval);
}

SPSocket.prototype.newSocket = function(addr, cb) {
    var socket = new WebSocket(addr);
    socket.binaryType = "arraybuffer";
    socket.onmessage = cb;
    return socket;
}

function ajax(method, dest, type, cb) {
    var req = new XMLHttpRequest();
    req.responseType = type;
    req.open(method, dest, true);
    req.onreadystatechange = function() {
        if (req.readyState == 4 && req.status == 200) {
            cb(req.response);
        }
    }
    req.send();
}

var graphs = [
    new TargetGraph("tcpping_graph",
        function(val) {
            return (val / 1000).toFixed() + " ms";
        }
    )
]

ajax("GET", "/api/options", "json", function(res) {
    console.log("Fetched option from /api/options.");
    graphs[0].setSeriesLabels(res.tcpping_options.addrs);
});

new SPSocket("ws://localhost:5002", function(message) {
    graphs[0].update(message.data);
});
