const hot_reloader = {
    init: function() {
        this.retry_id = undefined;
        this.connect(false);
    },
    cancel: function() {
        clearTimeout(this.retry_id);
    },
    connect: function(should_reload) {
        var socket = new WebSocket("ws://{hot_reload_host}:{hot_reload_port}/ws", "hot_reload");
        socket.onopen = function(event) {
            if (should_reload) {
                location.reload(true);
                //cancel the timer
                if (typeof this.timeoutID === 'number') {
                    this.cancel();
                }
            }
        }
        socket.onclose = function(event) {
            // Allow the last socket to be cleaned up.
            socket = null;

            // Set an interval to continue trying to reconnect
            // periodically until we succeed.
            this.retry_id = setTimeout(function() {
                hot_reloader.connect(true);
            }.bind(this), 5000, "Attempting to reconnect")
        }
        socket.onmessage = function(event) {
            {
                //cancel the timer
                if (typeof this.timeoutID === 'number') {
                    this.cancel();
                }
                socket.close(1000, "Hot reloading readme");
                location.reload(true);
            }
        }

    }

}

hot_reloader.init();
