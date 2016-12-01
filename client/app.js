'use strict';

const {h, render, Component} = window.preact;

const SENTINEL_ERROR = -2100000000;
const SENTINEL_NODATA = -2000000000;
const TARGET_KINDS = ['tcpping', 'httpdownload'];

class SPSocket {
    constructor(port, cb, interval) {
        if (!interval) {
            interval = 20000;
        }

        this.addr = 'ws://' + window.location.hostname + ':' + port;
        this.socket = this.newSocket(cb);

        setInterval(function() {
            if (this.socket.readyState > 1) {
                console.log('Reconnecting WebSocket...');
                this.socket = this.newSocket(cb);
            }
        }.bind(this), interval);
    }

    newSocket(cb) {
        var socket = new WebSocket(this.addr);
        socket.binaryType = 'arraybuffer';
        socket.onmessage = cb;
        return socket;
    }
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

function dateAxisFormatter(epochSeconds, gran, opts) {
    return Dygraph.dateAxisLabelFormatter(new Date(epochSeconds * 1000), gran, opts);
}

function dateFormatter(epochSeconds) {
    return Dygraph.dateString_(epochSeconds * 1000);
}

class Graph extends Component {
    constructor() {
        super();
        this.graph = null;
    }

    componentDidMount() {
        var gvFormatter = function(val, opts, seriesName) {
            if (seriesName == 'Time') {
                return dateFormatter(val);
            } else if (val == SENTINEL_ERROR) {
                return 'Error/timeout';
            } else if (val == SENTINEL_NODATA) {
                return 'No Data';
            } else {
                return this.props.valFormatter(val);
            }
        }.bind(this);

        this.graph = new Dygraph(
            document.getElementById('graph_' + this.props.kind),
            [[0]],
            {
                valueFormatter: gvFormatter,
                valueRange: [0, null],
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
                    this.update();
                }.bind(this)
            }
        );
    }

    update() {
        if (this.graph && !this.graph.isZoomed()) {
            console.log('Graph.update() executing actual update.');
            this.graph.updateOptions({
                labels: ['Time'].concat(this.props.options.addrs),
                isZoomedIgnoreProgrammaticZoom: true,
                valueRange: [0, this.props.max + 2],
                file: this.props.data
            });
        }
    }

    shouldComponentUpdate() {
        return this.graph == null;
    }

    render() {
        return h('div', {
            id: 'graph_' + this.props.kind,
            className: 'graph'
        });
    }
}

class Target extends Component {
    constructor(props) {
        super(props);
        this.state = {
            options: {},
            max: null
        };
    }

    componentDidMount() {
        ajax('GET', '/api/target/' + this.props.kind, 'json', function(res) {
            console.log('Fetched option for: ' + this.props.kind);
            this.setState({
                options: res
            });
        }.bind(this));
    }

    render() {
        return h('div', null, [
            h(Graph, {
                ref: (g) => {
                    g.update();
                },
                kind: this.props.kind,
                valFormatter: this.props.valFormatter,
                data: this.data,
                options: this.state.options,
                max: this.state.max
            })
        ]);
    }

    liveDataUpdate(nonce, arr) {
        if (nonce != this.state.options.nonce) {
            console.log("Mismatched nonce!");
        }

        console.log(arr);
        if (!this.data) {
            this.data = [];
        }

        this.data.push(arr);

        var curMax = this.state.max;
        for (var i = 1; i < arr.length; i++) {
            if (arr[i] > curMax) {
                curMax = arr[i];
            }
        }

        this.setState({
            max: curMax
        });
    }
}

class App extends Component {
    constructor() {
        super();
        this.targets = {};
    }

    handleSocketMessage(message) {
        console.log('Received WebSockets message.');
        var buf = message.data;
        var raw = new Int32Array(buf);

        var kind_id = raw[0];
        var nonce = raw[1];
        var arr = raw.slice(2);
        this.targets[TARGET_KINDS[kind_id]].liveDataUpdate(nonce, arr);
    }

    componentDidMount() {
        ajax('GET', '/api/config/ws_port', 'text', function(port_str) {
            new SPSocket(port_str, this.handleSocketMessage.bind(this));
        }.bind(this));
    }

    render() {
        return h(Target, {
            ref: (t) => {
                this.targets['tcpping'] = t;
            },
            kind: 'tcpping',
            valFormatter: function(val) {
                return (val / 1000).toFixed() + " ms";
            }
        });
    }
}

render(h(App), document.getElementById('app'));
