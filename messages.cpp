#include "messages.h"

size_t sensor_message(char* buffer, uint16_t length, const char* name, const int reading) {
    return snprintf(buffer, length, "%s:%i", name, reading);
}
