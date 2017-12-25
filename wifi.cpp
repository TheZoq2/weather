#include "wifi.h"

//#include <Arduino.h>
#include "ardumacro.h"

const uint8_t REPLY_BUFFER_SIZE = 10;

void wifi_line_end() {
    SERIAL1_WRITE("\r");
    SERIAL1_WRITE("\n");
}

bool buffer_matches(char buffer[REPLY_BUFFER_SIZE], const char* key) {
    auto key_len = strlen(key);
    if(key_len > REPLY_BUFFER_SIZE) {
        DEBUG_PLN("Warning: key_len is > REPLY_BUFFER_SIZE");
        return false;
    }

    for(uint8_t i = 0; i < key_len; ++i) {
        if(key[i] != buffer[REPLY_BUFFER_SIZE - key_len + i]) {
            return false;
        }
    }

    return true;
}

WifiReply wait_for_wifi_result() {
    auto done = false;

    auto result = WifiReply::Err;

    char buffer[REPLY_BUFFER_SIZE];

    while(!done) {
        if(SERIAL1_AVAILABLE()) {
            for(uint8_t i = 0; i < REPLY_BUFFER_SIZE-1; ++i) {
                buffer[i] = buffer[i+1];
            }
            buffer[REPLY_BUFFER_SIZE - 1] = SERIAL1_READ();

            DEBUG_P(buffer[REPLY_BUFFER_SIZE-1]);
        }

        if(buffer_matches(buffer, "\r\nOK") || buffer_matches(buffer, "\r\nSEND OK")) {
            result = WifiReply::Ok;
            done = true;
        }
        else if(buffer_matches(buffer, "\r\nERROR")) {
            result = WifiReply::Err;
            done = true;
        }
    }

    return result;
}

WifiReply wait_for_prompt() {
    auto done = false;
    auto result = WifiReply::Err;
    while(!done) {
        if(SERIAL1_AVAILABLE()) {
            auto read = SERIAL1_READ();
            DEBUG_W(read);
            if (read == '>') {
                result = WifiReply::Ok;
                done = true;
            }
        }
    }
    return result;
}

WifiReply setup_wifi() {
    //Enter client mode
    SERIAL1_PRINT("AT+CWMODE=3");
    wifi_line_end();

    try_wifireply(wait_for_wifi_result());

    // Connect to AP
    SERIAL1_PRINT("AT+CWJAP=\"");
    SERIAL1_PRINT(SSID);
    SERIAL1_PRINT("\",\"");
    SERIAL1_PRINT(PASSWORD);
    SERIAL1_PRINT("\"");
    wifi_line_end();

    try_wifireply(wait_for_wifi_result());

    //Enter client mode
    SERIAL1_PRINT("AT+CIPMUX=1");
    wifi_line_end();

    return wait_for_wifi_result();
}

WifiReply send_data(const char* data) {
    auto data_length = strlen(data);

    SERIAL1_PRINT("AT+CIPSTART=1,\"TCP\",\"");
    SERIAL1_PRINT(SERVER_IP);
    SERIAL1_PRINT("\",2000");
    wifi_line_end();
    try_wifireply(wait_for_wifi_result());

    SERIAL1_PRINT("AT+CIPSEND=1,");
    SERIAL1_PRINT(data_length);
    wifi_line_end();
    try_wifireply(wait_for_wifi_result());
    try_wifireply(wait_for_prompt());
    SERIAL1_PRINT(data);
    try_wifireply(wait_for_wifi_result());

    SERIAL1_PRINT("AT+CIPCLOSE=1");
    wifi_line_end();
    return wait_for_wifi_result();
}
