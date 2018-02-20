unexport CARGO_INCREMENTAL

TARGET_NAME=thumbv7em-none-eabi

all:
	make build
	make gdb

build:
	xargo build

openocd:
	openocd -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg

blue_openocd:
	openocd -f bluepill.cfg

gdb:
	arm-none-eabi-gdb target/${TARGET_NAME}/debug/cortex-m-quickstart

gdb_release:
	arm-none-eabi-gdb target/${TARGET_NAME}/release/cortex-m-quickstart
