
var socket = new WebSocket("ws://localhost:5002");
socket.onmessage = function(message) {
    var e = document.createElement("p");
    e.innerHTML = message.data;
    document.body.appendChild(e);
    console.log(message.data);
}
