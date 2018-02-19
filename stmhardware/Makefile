unexport CARGO_INCREMENTAL

all:
	make build
	make gdb

build:
	xargo build

openocd:
	openocd -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg

gdb:
	arm-none-eabi-gdb target/thumbv7em-none-eabihf/debug/cortex-m-quickstart

gdb_release:
	arm-none-eabi-gdb target/thumbv7em-none-eabihf/release/cortex-m-quickstart
