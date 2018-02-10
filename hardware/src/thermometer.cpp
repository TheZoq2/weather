#include "thermometer.h"

#include <Arduino.h>

void setup_thermometer() {
    pinMode(THERMOMETER_GND_PIN, OUTPUT);
    pinMode(THERMOMETER_VIN_PIN, OUTPUT);
    digitalWrite(THERMOMETER_GND_PIN, LOW);
    digitalWrite(THERMOMETER_VIN_PIN, HIGH);
}

float read_thermometer() {
    return (analogRead(THERMOMETER_ANALOG_ID) / 1024.0) * SUPPYLY_VOLTAGE * 100;
}

