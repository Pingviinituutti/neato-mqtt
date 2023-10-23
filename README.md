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

MQTT messages follow this structure, serialized as JSON:

```
struct MqttDevice {
    pub id: String,
    pub name: String,
    pub power: Option<bool>,
    pub brightness: Option<f32>,
    pub cct: Option<f32>,
    pub color: Option<Hsv>,
    pub transition_ms: Option<f32>,
    pub sensor_value: Option<String>,
}
```

Example light state:

```
{
  "id": "d68b5135-0e71-4333-ad3d-07b635144422",
  "name": "Office",
  "power": null,
  "brightness": 0.5,
  "cct": null,
  "color": {
    "neato": 31.238605,
    "saturation": 0.7411992,
    "value": 1
  },
  "transition_ms": null,
  "sensor_value": null
}
```

Example sensor state (2nd dimmer switch button pressed in):

```
{
  "id": "a8f6b7e3-80a1-45ee-9af6-ef9b6204c72d",
  "name": "Office switch button 2",
  "power": null,
  "brightness": null,
  "cct": null,
  "color": null,
  "transition_ms": null,
  "sensor_value": "true"
}
```
