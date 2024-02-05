
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
	    - **Attention: Pins 8, 10 and 12 are GND. In future versions, only pin 12 will remain GND. Pins 8 and 10 will be +24V.**
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

## Cost
I've ordered 10 PCBs from JLCPCB, with SMD components included and soldered, but without any through-hole components. I payed about 100 Euro incl. tax and shipping, so around 10 Euros per PCB. This does not include the ESP32-C6 (3.60 Euro) because they didn't have it in stock, nor the VEML6040 Light sensor (0.90 Euro) because I forgot to place it. So the actual price would be 14.50 Euro per board.

When ordering 100 pieces, and with some very minor tweaks in v1.1 to reduce the assembly fees, price per PCB incl. SMD assembly will drop to 8.10 Euro incl. ESP and light sensor.

The through hole components are 4.30 Euro per board and will drop top 2.70 for v1.1.

So the full v1.1 board will be around 10.80 Euro.

## Known problems for v1
_Should be converted into issues soon, but it's quicker to just list them here._

### High severity
_No known fix_

* [ ] The inductor of the 5V buck converter makes an audible noise, especially when the presence sensor is connected

### Medium severity
_Prevent the board from working, but can easily be fixed manually_

* [x] Wrong footprint for R11 and R15, so they are not assembled. Because they are not really needed, you can bridge them with a drop of solder.
* [x] Pin 1 of VR1 (Enable of the 3.3V voltage regulator) needs to be connected to 5V. You can place a very short wire or even a drop of of solder over to Pin 2.
* [x] Pin 3 of the ESP (Enable) module needs a pull up resistor. You can hand-solder one to the bottom of J8, or instead of J8 if you don't need the connector.
* [x] Light sensor is no placed, and it's unknown if it would work if placed

### Low severity
_Would be nice to fix these, but it's ok as it is_

* [ ] The antenna of the ESP extends over the rectangular board footprint. That was a deliberate design choice, but I regret it.
* [x] Mounting holes are too small for M3 screws.
* [ ] An additional mounting hole at the top right would be nice.
* [ ] USB data lines are length-matched and work just fine (it's just USB 2.0), but violate many rules:
   * not impedance-controlled (I tried, but used totally wrong numbers)
   * opposite ground plane is not continuous
   * four vias when it could be done with zero or two
* [ ] Thick wires have tiny vias
* [ ] Net `Pres_B` of the presence sensor is shifted down to 3.3V but not connected to the ESP or anything else
* [ ] Would be nice to have some small LEDs on board
	* Power LEDs that light up as long as there is 24V, 5V and 3,3V (should be very dim)
	* 1 LED per PWM channel to test it without an LED module attached (common +24V could be switched with a jumper)
* [ ] It would also be nice to support USB Power Delivery, so that the LEDs could light up when only USB is connected, for easier development. PD does not support 24V, only 20V, but the LEDs that I tested are ok with that.
* [ ] use an SMD version of the temperature sensor because it's much cheaper