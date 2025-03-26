<!--
SPDX-FileCopyrightText: 2024 Lena Schimmel <mail@lenaschimmel.de>
SPDX-License-Identifier: CC-BY-SA-4.0

[besteLampe!](https://lenaschimmel.de/besteLampe!) © 2024 by [Lena Schimmel](mailto:mail@lenaschimmel.de) is licensed under [CC BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/?ref=chooser-v1)
-->

## LED Module A2
Like module A, this board fits into some Steinel Lamp enclosures, but is not compatible with the existing hardware in that lamp, so it is not intended as a simple replacement / upgrade for that lamp.

 Differences to Module A:
 - Standard FR4-PCB instead of aluminum
 - Two layers instead of one
 - Wider traces, no more squeezing below resistors
 - Through-Hole connector, which is much cheeper
 - Alternative temperature sensor / EEPROM
 - UV LED channel

This has not been assigned the name "A2" instead of an increased version number for "A", because the single-layer aluminum process might still have advantages and evolve.

**Note: Both the I2C sensor and the 7th channel with UV LEDs is not fully compatible with the pin out of the Main Module v1. Adjustment to the Main Module are needed to actually use these two features.**