<!--
SPDX-FileCopyrightText: 2024 Lena Schimmel <mail@lenaschimmel.de>
SPDX-License-Identifier: CC-BY-SA-4.0

[besteLampe!](https://lenaschimmel.de/besteLampe!) © 2024 by [Lena Schimmel](mailto:mail@lenaschimmel.de) is licensed under [CC BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/?ref=chooser-v1)
-->

## led-benchmark
This project is used to drive LEDs using different driver circuits and PWM generators, and benchmark several properties.

Measured values:
 - Brightness (VEML6040)
 - Color temperature (VEML6040)
   - *I do not yet have the correct code to compute the color temperature, so I will save the four sensor values (RGBW) separately for later processing*
 - Power consumption (INA219)
 - Temperature (heat) of the LEDs and/or driver (TMP1075 and/or some DS18B20 variant)

The LEDs may differ by:
 - type
 - count (usually in series)
 - cooling

The driver electronic may differ by:
 - model of the CCD IC
 - inductor
 - CCD switching frequency
 - (desired) current at 100% duty cycle
 - input voltage

The software changes these parameters:
 - PWM frequency
 - duty cycle

It's obvious that I can't test every possible combination of those parameters, especially not those that require me to build different driver boards.

### Needed hardware
- LED controlled by PWM signal from GPIO 11
- I2C Bus with SDA on GPIO 23 and SCL on GPIO 22
- INA219 on I2C with address 64 (sadly hardcoded in the fork that I use)
  - The measured current is completely wrong. Not sure if wrong calibration, other software error or I connected my wires incorrectly
- TMP1075 on I2C with address 77