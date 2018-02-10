# Hardware

This is the arduino based sensor-reading code which performs actual measurements
and sends them to the server.

## Setup:
Create a `paths.mk` file containing an enviroment variable pointing to your arduino
installation location. For example:

```bash
ARDUINO_DIR=~/bin/arduino-1.6.12
```

Create a `wifidetails.h` file in `src/`

```C++
const auto SSID = "<Your wifi network name>";
const auto PASSWORD = "<Your wifi network password>";

const auto SERVER_IP = "<IP address or URL of the server>"
```


