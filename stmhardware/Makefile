unexport CARGO_INCREMENTAL

build:
	xargo build

openocd:
	openocd -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg

gdb:
	arm-none-eabi-gdb target/thumbv7em-none-eabihf/debug/cortex-m-quickstart
