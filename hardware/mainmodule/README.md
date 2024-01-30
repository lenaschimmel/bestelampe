
# Main module 

<img src="/assets/schematic.jpg" width="49%"/><img src="/assets/layout.jpg" width="49%"/>

## Hardware features / components
- Standard 2-layer PCB
- ESP32-C6 MCU
- Supports four wireless protocols (only one at a time, except Wifi and Bluetooh which can co-exist)
  - Wifi
  - Bluetooth
  - Zigbee
  - Thread
- 6 MOSFET-driven PWM channels with flexible frequency / bit depth
  - e.g. 13 bits = 8192 levels at 5000 Hz
  - or 15 bits = 32,768 levels at 1250 Hz
  - optional 4 bit temporal dithering, increasing the levels by a factor of 16 without reducing the frequency
- Connectors
	- J1: multi-purpose USB-C connector
		- flash new firmware, even when current firmware is broken
		- view logs
		- power the board (Powers everything but LEDs. You may combine USB power and DC power, and switch between both at runtime)
	- J2: LED module. Supports temperature sensor(s) on the LED module.
	- J3: Extension module. Usually contains a secondary MCU and additional LEDs and/or hardware.
	- J4: Power. DC Input 6V to 24V.
		- Must be the same as LED voltage.
		- Generates 5V (buck converter) and 3,3V (linear voltage regulator) for internal and external peripherals.
	- J5: Temp. Attach additional temperature sensor(s) here and/or on the LED module and/or extension module.
	- J6: Presence sensor. Made for seed studio model MR24HPC1, may work with others.
	    - Provided 5V power to the sensor.
		- TX is shifted up from 3,3V to 5V
		- RX, A and B are shifted down form 5V to 3,3V
		- B is not connected to the MCU
	- J7: Neopixel. You can connect addressable LEDs here. Only 5V supported on this connector. Modules on J2 and J3 also get the control signal, and get 24V to power LEDs.
	- J8: AUX. Used for debugging, testing, etc.
	- J9 and J10: Light sensor. The VEML6040 sensor is directly connected on the PCB, but you could break off or cut off the part with the sensor and use these pins to connect it over wire.
- 1-wire bus for temperature sensors and other devices
	- onboard 12 bit temperature sensor between Mosfet and capacitors
	- bus is also connected to J2, J3, J5 and J8