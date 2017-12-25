SOURCES=wifi.cpp messages.cpp

TEST_SOURCE=test.cpp

upload:
	make -f arduino.mk
	make -f arduino.mk reset

monitor:
	make -f arduino.mk monitor


test:
	g++ ${SOURCES} ${TEST_SOURCE} -g
	gdb a.out
