# Weather

![Very WIP hardware](hardware.jpg)

This is a work-in-progress weather station that comprised of 3 parts. A web frontend
written in elm, a server for collecting the data written in rust, and the actual
sensor hardware. Originally, the sensor hardware was written in C++ for a teensy LC
but due to various circumstances I decided to replace it with a
[blue pill](http://wiki.stm32duino.com/index.php?title=Blue_Pill) which is programmed
in rust. The original arduino code can be found in `hardware` while the blue pill code
can be found in `stmhardware`.

The project also contains a `models/` directory which contains models for 3d
printing the hardware

## Planned sensors

The current plan is to have the weather station meassure temperature, humidity, 
air pressure and wind speed. If I can come up with a good way of doing it, I would
also like to add a rain sensor and a wind direction sensor.

