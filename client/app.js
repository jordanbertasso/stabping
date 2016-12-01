'use strict';

const {h, render, Component} = window.preact;

const SENTINEL_ERROR = -2100000000;
const SENTINEL_NODATA = -2000000000;

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
                    if (!this.graph.isZoomed()) {
                        this.updateDrawnGraph();
                    }
                }.bind(this)
            }
        );
    }

    updateDrawnGraph() {
        console.log('updateDrawnGraph() called.');
        this.graph.updateOptions({
            labels: ['Time'].concat(this.props.options.addrs),
            isZoomedIgnoreProgrammaticZoom: true,
            valueRange: [0, this.props.max],
            file: this.props.data
        });
    }

    update() {
        if (this.graph && !this.graph.isZoomed()) {
            this.updateDrawnGraph();
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

    liveDataUpdate(arr) {
        if (!this.data) {
            this.data = [];
        }

        this.data.push(arr);

        var curMax = this.max;
        for (var i = 1; i < new_data.length; i++) {
            if (new_data[i] > curMax) {
                curMax = new_data[i];
            }
        }
        this.max = curMax;
    }
}

class App extends Component {
    handleSocketMessage(message) {
        var buf = message.data;
        console.log('Received WebSockets message.');
    }

    componentDidMount() {
        ajax('GET', '/api/config/ws_port', 'text', function(port_str) {
            new SPSocket(port_str, this.handleSocketMessage);
        }.bind(this));
    }

    render() {
        return h(Target, {
            kind: 'tcpping',
            valFormatter: function(val) {
                return (val / 1000).toFixed() + " ms";
            }
        });
    }
}

render(h(App), document.getElementById('app'));
