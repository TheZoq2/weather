#pragma once

#include "wifidetails.h"

enum class WifiReply {
    Ok,
    Err
};

#define try_wifireply(expr) if(expr == WifiReply::Err){return WifiReply::Err;}



void wifi_line_end();

WifiReply wait_for_wifi_result();

WifiReply setup_wifi();

WifiReply send_data(const char* data);

WifiReply wifi_reset();

