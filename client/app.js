function date_formatter(epoch_seconds) {
    return Dygraph.dateString_(epoch_seconds * 1000);
}

function tcpping_value_formatter(val, opts, seriesName) {
    if (seriesName == "Time") {
        return date_formatter(val);
    } else {
        return (val / 1000).toFixed() + " ms";
    }
}

var tcpping_graph = new Dygraph(
    document.getElementById("tcpping_graph"),
    [[0]],
    {
        valueFormatter: tcpping_value_formatter,
        axes: {
            x: {
                axisLabelFormatter: date_formatter
            },
            y: {
                axisLabelFormatter: tcpping_value_formatter
            }
        }
    }
);

var tcpping_data = null;

function tcpping_data_update(buf) {
    if (tcpping_data == null) {
        tcpping_data = [];
    }

    var new_data = new Uint32Array(buf);
    tcpping_data.push(new_data);

    tcpping_graph.updateOptions({
        file: tcpping_data
    });
}

var options_req = new XMLHttpRequest();
options_req.responseType = "json";
options_req.open("GET", "/api/options", true);
options_req.onreadystatechange = function() {
    if (options_req.readyState == 4 &&
        options_req.status == 200) {
        var options = options_req.response;
        console.log("Fetched option from /api/options.");
        tcpping_graph.updateOptions({
            labels: ["Time"].concat(options.tcpping_options.addrs)
        });
    }
}
options_req.send();

var socket = new WebSocket("ws://localhost:5002");
socket.binaryType = "arraybuffer";
socket.onmessage = function(message) {
    tcpping_data_update(message.data);
}
