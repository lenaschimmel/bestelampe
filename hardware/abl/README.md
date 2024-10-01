<!--
SPDX-FileCopyrightText: 2024 Lena Schimmel <mail@lenaschimmel.de>
SPDX-License-Identifier: CC-BY-SA-4.0

[besteLampe!](https://lenaschimmel.de/besteLampe!) © 2024 by [Lena Schimmel](mailto:mail@lenaschimmel.de) is licensed under [CC BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/?ref=chooser-v1)
-->
## besteLampe! Access Balcony Lamp (ABL)
The Access Balcony Lamp (short ABL, German "Laubengang-Lampe") will be the first device of the besteLampe! series to actually be manufactured.

## Component overview
It consists of these parts:
 - Power Module
   - [Power PCB](./power/)
 - Lamp Module 
   - [LED PCB](../ledmodule/LED_Module_E/)
   - [Controller PCB](./Controller/)
 - Connectors between the main PCBs
   - [Rigid connector](./Conductor/)
   - [Flex connector](./Flex/)
 - Enclosure
   - Not yet online

_(The names, labels and paths of those modules are currently inconsistent and will be renamed soon.)_

![Simplified diagram of modules and components](./ABL%20Components.png)

## I2C addresses
All modules share a common I2C bus. In the above diagram, the I2C components are highlighted by violet text.

Most devices support multiple addresses, which are selected via address pins. Some of them recognize 4 states per pin: high, low, SDA and SDC. Using static high and low for address selection seems to be more robust.

The INA219 power sensor on the power board serves an additional purpose: the address it reacts to indicates if a 20W or 30W power supply has been installed in the power module.

 - Power Module
    - 64 = 1000000: INA219 (in case of 20W power supply)
    - 65 = 1000001: INA219 (in case of 30W power supply)
    - 79 = 1001111: TMP1075
 - LED Module
    - 75 = 1001011: TMP1075
    - 80 = 1010000: 24AA02 (May react to any address from 80 to 87)
 - Controller Module
    - 16 = 0010000: VEML6040
    - 67 = 1000011: FLX6408 (3.3V)
    - 68 = 1000100: FLX6408 (5V)
    - 69 = 1000101: INA219
 - Feather
    - 54 = 0110110:	Battery monitor (Adafruit ESP-C6)