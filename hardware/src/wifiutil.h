#pragma once

#include <stdint.h>

const uint8_t REPLY_BUFFER_SIZE = 10;

bool buffer_matches(char buffer[REPLY_BUFFER_SIZE], const char* key) {
    auto key_len = strlen(key);
    if(key_len > REPLY_BUFFER_SIZE) {
        Serial.println("Warning: key_len is > REPLY_BUFFER_SIZE");
        return false;
    }

    for(size_t i = 0; i < key_len; ++i) {
        if(key[i] != buffer[REPLY_BUFFER_SIZE - key_len + i]) {
            return false;
        }
    }

    Serial.println("");

    return true;
}
