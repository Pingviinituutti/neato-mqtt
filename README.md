# neato-mqtt

This program synchronizes your Neato Botvac vacuum robots with an MQTT broker.

It uses the Nucleo API to fetch available robots, read states and send commands. 
The Nucleo API is documented on https://developers.neatorobotics.com/api/nucleo.

## Setup


### Quick start

- Take note of the username and password you use for your Neato account.
- Copy `Settings.example.toml` to `Settings.toml`.
- Edit `Settings.toml` with values matching your setup.
- Try running neato-mqtt with `cargo run`. If your bridge runs recent enough firmware, the program should now launch without errors.

Now you should be able to view your Neato Botvac robots on the MQTT broker, via e.g. [MQTT Explorer](http://mqtt-explorer.com/).

To make the robot(s) clean and other actions, simply publish a message under `home/devices/neato/{id}/set`:

``` json
{
  "action": "StartCleaning",
}
```

If you publish your action under `home/devices/neato/set`, the action will be sent to all robots under `home/devices/neato/` (or based on what `topic` and `set_topic` settings you have in `Settings.toml`).

Available messages are listed on https://developers.neatorobotics.com/api/robot-remote-protocol/housecleaning. 
Note that the `action` value should start with an uppercase letter (TODO fix this).

### Setting Up Mosquitto 

- Ensure Docker is installed and running

- Run mosquitto (with authentication disabled) using:

  ```
  docker run -it -p 1883:1883 eclipse-mosquitto mosquitto -c /mosquitto-no-auth.conf
  ```

### Setting Up MQTT Explorer

- Install [MQTT Explorer](http://mqtt-explorer.com/)

- Connect to the MQTT broker by configuring the connection

  - Setting the protocol to mqtt://
  - Setting the host to localhost
  - Setting the port to 1883

## Topics

The default MQTT topics are as follows:

- `/home/devices/neato/{id}`: Current state of the device serialized as JSON
- `/home/devices/neato/{id}/set`: Sets state of the light to given JSON

## State messages

Example robot state in JSON:

```json
{
  "mac_address": "123456789012",
  "model": "BotVacD6Connected",
  "name": "Vacuum",
  "nucleo_url": "https://nucleo.neatocloud.com:4443",
  "serial": "12345678-123456789012",
  "state": {
    "alert": null,
    "error": null,
    "details": {
      "isCharging": true,
      "isDocked": true,
      "isScheduleEnabled": true,
      "dockHasBeenSeen": false,
      "charge": 99
    },
    "state": 1,
    "action": 0
  }
}
```
