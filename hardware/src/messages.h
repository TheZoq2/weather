#pragma once

#include "ardumacro.h"

/*
  Creates a json formated sensor message from a sensor name and a floating point value
  
  If the total length of the message is longer than length, it is cropped
*/
size_t sensor_message(char* buffer, uint16_t length, const char* name, const int reading);
