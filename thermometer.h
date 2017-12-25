#ifndef H_THERMOMETER
#define H_THERMOMETER

#include <stdint.h>

const uint8_t THERMOMETER_VIN_PIN = 23;
const uint8_t THERMOMETER_GND_PIN = 22;
const uint8_t THERMOMETER_ANALOG_ID = 7;
const float SUPPYLY_VOLTAGE = 3.3;

void setup_thermometer();
float read_thermometer();

#endif
