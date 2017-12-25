#This file must contain the following variables
# ARDUINO_DIR=path_to_arduino_home

include ./paths.mk

BOARD_TAG    = teensyLC
ARDUINO_LIBS = 

LOCAL_CPP_SRCS = messages.cpp wifi.cpp thermometer.cpp

CXXFLAGS += -DIS_ARDUINO

USB_TYPE = USB_SERIAL_HID

#include ../../Teensy.mk
include ~/Arduino/Arduino-Makefile/Teensy.mk
