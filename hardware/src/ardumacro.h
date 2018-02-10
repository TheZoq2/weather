#pragma once

#ifdef IS_ARDUINO
    #include <Arduino.h>

    #define DEBUG_P(expr) Serial.print(expr)
    #define DEBUG_PLN(expr) Serial.println(expr)
    #define DEBUG_W(expr) Serial.write(expr)

    #define SERIAL1_PRINTLN(expr) Serial1.println(expr)
    #define SERIAL1_PRINT(expr) Serial1.print(expr)
    #define SERIAL1_READ(expr) Serial1.read(expr)
    #define SERIAL1_WRITE(expr) Serial1.write(expr)
    #define SERIAL1_AVAILABLE() Serial1.available()
#else
    #include <iostream>
    #include <cstring>

    #define DEBUG_P(expr) std::cout << expr
    #define DEBUG_PLN(expr) std::cout << expr << std::endl
    #define DEBUG_W(expr) std::cout << expr

    #define SERIAL1_PRINTLN(expr) std::cout << expr << " [to Serial1]" << std::endl
    #define SERIAL1_PRINT(expr) std::cout << expr
    #define SERIAL1_READ(expr) '0'
    #define SERIAL1_WRITE(expr) std::cout << expr
    #define SERIAL1_AVAILABLE() true
#endif
