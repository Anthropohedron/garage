# Garage Door Monitoring and Control

This was developed to run on a Raspberry Pi 4B, but it could be tweaked to
run on pretty much anything supporting the Linux GPIO chardev interface.
This README is just about the code, but there's more information about
deployment and hardware and such in the [docs](docs/README.md).

## Controller

The `garage-control` executable is a web server that allows reading the
current garage door status (as determined by the monitor) and activating
the garage door opener. It is currently hardcoded to listen on 127.0.0.1
port 8080. It logs program start, program exit, and when the garage door
opener is activated. It logs to syslog under the `garagemon` process name.

## Monitor

The `garage-monitor` executable waits for GPIO events on pins connected to
sensors, one activated when the door is fully open and the other activated
when the door is fully closed. It logs program start, program exit, and the
new state of the garage door whenever an event changes it. It logs to
syslog under the `garagemon` process name. It also writes to a file in
`/var/run` for the controller to read.

## Future Work

1. Add command line arguments and/or a config file to configure various
   values that are currently hardcoded (e.g. GPIO lines and chip, listen
   host and port)
1. Use proper logging instead of logging directly to syslog
1. Use some better IPC mechanism (Unix domain socket?) to share door state
   from the monitor to the controller
1. Include documentation of how pins should be connected to hardware
