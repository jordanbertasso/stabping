'use strict';

const {h, render, Component} = window.preact;

const SENTINEL_ERROR = -2100000000;
const SENTINEL_NODATA = -2000000000;
const TARGET_KINDS = [
    {
        name: 'tcpping',
        pretty_name: 'TCP Ping',
        valFormatter: function(val) {
            return (val / 1000).toFixed() + ' ms';
        }
    }
    /*
    {
        name: 'httpdownload',
        pretty_name: 'HTTP Download',
        valFormatter: function(val) {
            return 'NOT YET IMPLEMENTED';
        }
    }
    */
];

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

function ajax(method, dest, type, success, error, data) {
    var req = new XMLHttpRequest();
    req.responseType = type;
    req.open(method, dest, true);
    req.onreadystatechange = function() {
        if (req.readyState == 4) {
            if (req.status == 200) {
                if (success) {
                    success(req.response);
                }
            } else {
                if (error) {
                    error(req);
                }
            }
        }
    }
    req.send(data);
}

function currentTime() {
    return Math.floor(new Date() / 1000);
}

var timeLoaded = currentTime();
function hoursBack(hours) {
    if (hours == -2) {
        // All
        return 0;
    } else if (hours == -1) {
        // Since Load
        return timeLoaded;
    } else {
        // Some number of hours
        return currentTime() - (hours * 3600);
    }
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
            this.base,
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
                isZoomedIgnoreProgrammaticZoom: true,
                zoomCallback: function (lowerDate, upperDate, yRanges) {
                    this.update();
                }.bind(this)
            }
        );
    }

    update() {
        if (this.graph && !this.graph.isZoomed()) {
            console.log('Graph.update() executing actual update for ' + this.props.kind.name);
            var h = hoursBack(this.props.preset);
            var dateWindow = h == 0 ? null : [h, this.props.data.slice(-1)[0][0]];

            this.graph.updateOptions({
                labels: ['Time'].concat(this.props.options.addrs),
                isZoomedIgnoreProgrammaticZoom: true,
                valueRange: [0, this.props.max + 2],
                dateWindow: dateWindow,
                file: this.props.data
            });
        }
    }

    shouldComponentUpdate() {
        return false;
    }

    render() {
        return h('div', {
            className: 'graph'
        });
    }
}

class Target extends Component {
    constructor(props) {
        super(props);
        this.state = {
            options: {},
            max: null,
            leftLimit: currentTime(),
            preset: 1,
            rollingSelection: 0
        };
    }

    componentDidMount() {
        ajax('GET', '/api/target/' + this.props.kind.name, 'json', function(res) {
            console.log('Fetched option for: ' + this.props.kind.name);
            this.setState({
                options: res
            });

            setTimeout(function() {
                this.persistentDataRetrieve(this.state.preset);
            }.bind(this), 300);
        }.bind(this));
    }

    persistentDataRetrieve(hoursPreset) {
        if (hoursPreset == 0) {
            return;
        }

        var leftTarget = hoursBack(hoursPreset);
        var leftLimit = this.state.leftLimit;
        var elementLength = this.state.options.addrs.length + 1;
        var nonce = this.state.options.nonce;

        if (leftTarget < leftLimit) {
            ajax('POST', '/api/target/' + this.props.kind.name, 'arraybuffer', function(res) {
                if (nonce == this.state.options.nonce) {
                    var raw = new Int32Array(res);
                    var newData = [];

                    var curMax = this.state.max;
                    for (let j = 0; j < raw.length; j += elementLength) {
                        var arr = raw.slice(j, j + elementLength);
                        newData.push(arr);
                        for (let i = 1; i < arr.length; i++) {
                            if (arr[i] > curMax) {
                                curMax = arr[i];
                            }
                        }
                    }

                    if (this.data) {
                        this.data = newData.concat(this.data);
                    } else {
                        this.data = newData;
                    }

                    console.log(this.data);
                    this.setState({
                        max: curMax,
                        leftLimit: leftTarget
                    });
                } else {
                    console.log('Mismatched nonce in persistent data retrieve!');
                }
            }.bind(this), function(err) {
                console.log('Failed to retrieve persistent data for range ' + leftTarget + ' to ' + leftLimit);
            }.bind(this), JSON.stringify({
                nonce: this.state.options.nonce,
                lower: leftTarget,
                upper: leftLimit
            }));
        }
    }

    onPresetChange(evt) {
        console.log('preset is now: ' + evt.target.value);
        this.persistentDataRetrieve(evt.target.value);
        this.setState({preset: evt.target.value});
    }

    onRollingSelectionChange(evt) {
        console.log('rollingSelection is now: ' + evt.target.value);
        this.setState({rollingSelection: evt.target.value});
    }

    render() {
        return h('div', {
            className: 'graph-container'
        }, [
            h('h2', null, [this.props.kind.pretty_name]),
            h(Graph, {
                ref: (g) => {
                    g.update();
                },
                kind: this.props.kind,
                valFormatter: this.props.valFormatter,
                data: this.data,
                options: this.state.options,
                max: this.state.max,
                preset: this.state.preset
            }),
            h('div', {
                className: 'graph-controls'
            }, [
                h('div', {className: 'control-group'}, [
                    h('div', {className: 'label'}, 'Base Time Interval'),
                    h('select', {
                        value: this.state.preset,
                        onChange: this.onPresetChange.bind(this)
                    }, [
                        h('option', {value: -1}, 'Since Load'),
                        h('option', {value: 0.25}, '15 Minutes'),
                        h('option', {value: 0.5}, '30 Minutes'),
                        h('option', {value: 1}, '1 Hour'),
                        h('option', {value: 3}, '3 Hours'),
                        h('option', {value: 6}, '6 Hours'),
                        h('option', {value: 12}, '12 Hours'),
                        h('option', {value: 24}, '1 Day'),
                        h('option', {value: 72}, '3 Days'),
                        h('option', {value: 168}, '1 Week'),
                        h('option', {value: 336}, '2 Weeks'),
                        h('option', {value: 744}, '1 Month'),
                        h('option', {value: -2}, 'All*')
                    ])
                ]),
                h('div', {className: 'control-group'}, [
                    h('div', {className: 'label'}, 'Rolling Average'),
                    h('select', {
                        value: this.state.rollingSelection,
                        onChange: this.onRollingSelectionChange.bind(this)
                    }, [
                        h('option', {value: 0}, 'None'),
                        h('option', {value: 1}, '1 Day'),
                        h('option', {value: 2}, '2 Days'),
                        h('option', {value: 3}, '3 Days'),
                        h('option', {value: 7}, '1 Week'),
                        h('option', {value: -1}, 'Custom')
                    ])
                ]),
                h('div', {className: 'control-group'}, [
                    h('div', {className: 'label'}, 'View Options'),
                    h('span', null, [
                        h('input', {type: 'checkbox'}),
                        'Checkbox'
                    ])
                ])
            ])
        ]);
    }

    liveDataUpdate(nonce, arr) {
        if (nonce != this.state.options.nonce) {
            console.log('Mismatched nonce! I have ' + this.state.options.nonce +
                        ' but this new one is ' + nonce);
            console.log(arr);
        }

        if (!this.data) {
            this.data = [];
        }

        this.data.push(arr);

        var curMax = this.state.max;
        for (let i = 1; i < arr.length; i++) {
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
        this.targets = new Array(TARGET_KINDS.length);
    }

    handleSocketMessage(message) {
        var buf = message.data;
        var raw = new Int32Array(buf);

        var kind_id = raw[0];
        var nonce = raw[1];
        var arr = raw.slice(2);
        this.targets[kind_id].liveDataUpdate(nonce, arr);
    }

    componentDidMount() {
        ajax('GET', '/api/config/ws_port', 'text', function(port_str) {
            new SPSocket(port_str, this.handleSocketMessage.bind(this));
        }.bind(this));
    }

    render() {
        var target_components = [];

        for (let i = 0; i < TARGET_KINDS.length; i++) {
            let kind = TARGET_KINDS[i];
            target_components.push(h(Target, {
                ref: (t) => {
                    this.targets[i] = t;
                },
                kind: kind,
                valFormatter: kind.valFormatter
            }));
        }

        return h('div', null, target_components);
    }
}

render(h(App), document.body);
