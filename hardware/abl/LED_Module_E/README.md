<!--
SPDX-FileCopyrightText: 2024 Lena Schimmel <mail@lenaschimmel.de>
SPDX-License-Identifier: CC-BY-SA-4.0

[besteLampe!](https://lenaschimmel.de/besteLampe!) © 2024 by [Lena Schimmel](mailto:mail@lenaschimmel.de) is licensed under [CC BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/?ref=chooser-v1)
-->

## LED Module E + G
The former LED Module E has been split into two separate PCBs:
 - Module E (classic 4--layer PCB with driver logic, MOSFETs and series resistor)
 - Module G (1-layer copper-core PCB with actual LEDs and temperature sensor)

In conjuction, they feature 10 small, high-powered Cree XLamp Element LEDs places close to one another along an edge, and some of the electronics needed to power them.

The split into separate PCBs is very recent, and the files have not yet been completely reorganized. Module G is still in this directory, and its files are still named "LED_Module_E_copper". They will move into their own directory soon.

### Parts on Module E
|   Reference                                                               |   Value             |   Datasheet                                                                                             |   Footprint                              |   Qty  |   DNP  |   Manufacturer_Part_Number  |   LCSC Part #  |   Purpose                         |
|---------------------------------------------------------------------------|---------------------|---------------------------------------------------------------------------------------------------------|------------------------------------------|--------|--------|-----------------------------|----------------|-----------------------------------|
|   C1,C2,C3                                                                |   100nF             |   ~                                                                                                     |   Capacitor_SMD:C_0402_1005Metric        |   3    |        |   CL05B104KO5NNNC           |   C1525        |   Decoupling capacitor            |
|   C4,C5                                                                   |   1000uF            |   ~                                                                                                     |   Capacitor_SMD:CP_Elec_10x7.9           |   2    |        |   VKME1301C102MV            |   C487313      |   Bulk capacitor                  |
|   J1                                                                      |   SFW20S-2STE1LF    |   https://cdn.amphenol-cs.com/media/wysiwyg/files/drawing/10172832.pdf                                  |   Own footprints:SFW20S2STE1LF_Own       |   1    |        |   SFW20S-2STE1LF            |   C3168412     |   Flex cable connector            |
|   J3                                                                      |   9159008603906     |   https://datasheets.kyocera-avx.com/9159-600.pdf                                                       |   9159008603906                          |   1    |        |   9159008603906             |   C22394454    |   Rigid board connector           |
|   Q1,Q2,Q3,Q4,Q5,Q6,Q7,Q8,Q11,Q12                                         |   Si3456DDV         |   https://www.vishay.com/docs/69075/si3456ddv.pdf                                                       |   Package_SO:TSOP-6_1.65x3.05mm_P0.95mm  |   10   |        |   SI3456DDV-T1-BE3          |   C6295864     |   MOSFET transistor               |
|   R1,R2,R3,R4,R5,R6,R7,R8,R9,R10,R11,R12,R13,R14,R15,R16,R17,R18,R19,R20  |   200mR             |   ~                                                                                                     |   Resistor_SMD:R_1020_2550Metric         |   20   |        |   RCWE1020R200FKEA          |   C4000899     |   Series resistor                 |
|   R21,R22,R23,R24,R25,R26,R27,R28,R31,R32,R33,R34,R35,R36,R37,R38         |   4700R             |   ~                                                                                                     |   Resistor_SMD:R_0402_1005Metric         |   16   |        |   0402WGF4701TCE            |   C25900       |   Pullup resistor                 |
|   R40,R41                                                                 |   2740R             |   ~                                                                                                     |   Resistor_SMD:R_0603_1608Metric         |   2    |        |   0603WAF2701T5E            |   C13167       |   Pullup resistor                 |
|   U2                                                                      |   74HC245           |   http://www.ti.com/lit/gpn/sn74HC245                                                                   |   Package_SO:TSSOP-20_4.4x6.5mm_P0.65mm  |   1    |        |   SN74AHCT245PWR            |   C10910       |   Bus transceivers / line driver  |
|   U3                                                                      |   24AA025E48T-I/OT  |   https://www.lcsc.com/datasheet/lcsc_datasheet_1810121540_Microchip-Tech-24AA025E48T-I-OT_C129895.pdf  |   Package_TO_SOT_SMD:SOT-23-6            |   1    |        |   24AA025E48T-I/OT          |   C129895      |   EEPROM                          |
|   U4                                                                      |   74LVC1G32         |   http://www.ti.com/lit/sg/scyt129e/scyt129e.pdf                                                        |   Package_TO_SOT_SMD:SOT-353_SC-70-5     |   1    |        |   74LVC1G32SE               |   C460530      |   Logic OR gate                   |

### Relation to previous LED modules
Version 1 of this module was partly based on the failed [LED Module D](../../ledmodule/LED_Module_D/), but with a lot of changes:

 - larger board
 - re-assinged all components to the three power rails
 - removed the TLC59711 as PWM generator
 - instead 8 separate inputs for PWM signals
 - two of the input signals are duplicated to power 2 LEDs each (but with separate MOSFETs and resistors)
 - more and larger mounting holes
 - improved thermal conductance (more thermal vias, etc.)
 - optimized traces
 - optional pull-ups for the I2C bus
 - solder jumper to directly tie the alert signal of the temperature sensor to the enable input of the driver
 - added test pads
 - added bulk capacitor
 - switched to 100% SMD parts
 - LED voltage can be supplied via USB-C or via two pins of the Pin header (see below)

It was a bit larger than LED module D, but the placement of the LEDs was still the same, which proved to be a good fit for the linear half-bown reflector prototype.

Now this is version 2, which has a very different form factor. It's much larger, and the LED are on the long side instead of the short one.

### VLED
With single LEDs (parallel, in contrast to series / strings), a voltage around 4V would be a good choice for the supply voltage, so that the series resistors don't waste too much energy.

But it's surprisingly difficult to find a good source for 4V if you've got only 230V AC, 5V and 3.3V DC to work with. It seems much more straight-forward to use 5V and adjust the series resistors to it.

The CD74HCT244 needs 4.5 V to 5.5 V to operate, so it already had a 5V rail. The power rail for the LEDs was historically called 4V, and even though it's lablled `VLED` on the silkscreen now, the nets are still called 4V in Kicad. Both rails are still separate, just in case I'll find a good 4V source in the near future.

### Temperature Alert
Usually, the MCU should monitor the temperature via the TMP1075 sensor, and reduce the LED power before they get too hot. Via I2C, the MCU can read the excact temperature value.

The MCU can also set an alert threshold on the sensor, so that it will trigger its ALERT pin when the LEDs get too hot.

As an extra safety measure, the ALERT net can be linked directly to the ENABLE net via the solder jumper JP1. If the MCU has crashed, or for some other reason is not able to react to the rising temperature, this should turn off the LEDs without the MCUs intervention. This feature is untested, and the jumper is open by default.

### EEPROM addressing issue
Version 2.0 of the LED board contained an 24AA02UID, which will react to any address from 80 to 87. This prevents the usage of multiple EEPROMs on one bus. Newer versions of the LED Module and Power Module use individually addressable 24AA02**5**UID (note the slightly different part number). In software, you should first check if an EEPROM on address on 87 is present. If yes, it's on the LED Module and you can't detect / use an EEPROM on the Power Module. If no, there *should* be an EEPROM at 81 on the LED Module, and there *could* be another on on 82 on the Power module.

### Rendering
![KiCad rendering of the PCB, as of 2024-10-24](../../../assets/rendering_abl_led.jpg)