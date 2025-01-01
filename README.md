<!--
SPDX-FileCopyrightText: 2024 Lena Schimmel <mail@lenaschimmel.de>
SPDX-License-Identifier: CC-BY-SA-4.0

[besteLampe!](https://lenaschimmel.de/besteLampe!) © 2024 by [Lena Schimmel](mailto:mail@lenaschimmel.de) is licensed under [CC BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/?ref=chooser-v1)
-->

<!-- *Apart from this README, most documentation and notes are in German. If this project becomes more than a proof-of-concept, all important parts the documentation will be translated to English. The name will remain German, as it should be understandable to English speakers as well.* -->

![Header image with PCB closeup](assets/header.jpg)

## besteLampe!
**Most lamps suck. This is an open source hardware and software project to create not only a better lamp, but the best lamp(s). Or *"besteLampe(n)!"* as we say in German.**

Of course, there are **many** kinds of lamps, and for some use cases, better alternatives exist. This one is created with two main use cases in mind:
 - indoor lamp for general room lighting and as a sunrise alarm clock 
 - outdoor lamp in front of apartments (called ABL, see below)

The first case involves more direct, manual control of a single lamp, and the second case is more about automatic control (time based, responding to movement, linked with nearby lamps). Anyway, the lamp in each use case will sometimes be operated *just like in the other use case*, so both have mostly the same requirements, just with different priorities.

## Structure of this repository
This project started out with a lot of experimental modules, see [experiments.md](./experiments.md) for information about those.

Current development is focused on the Access Balcony Lamp (ABL), see [hardware/abl](./hardware/abl/) for more information.

Development on indoor lamps may continue once the ABL is finished.

The repository is currently undegoing some restructuring to make it clearer which parts are outdated, active or for future applications.

## What makes ~~a good~~ the best lamp?
These are the design goals of *besteLampe!* 

 - **Free and Open** • open hardware, open software, open protocols
 - **State-of-the-art dimming** • Flicker free, smooth dimming, extremely wide contrast ratio (a.k.a "dim to zero"), independent dimming of color and brightness. (See  ["The problem with driving LEDs with PWM"](https://codeinsecurity.wordpress.com/2023/07/17/the-problem-with-driving-leds-with-pwm/) by Graham to understand what's so hard about that.)
 - **Premium color control** • Use as many color channels as needed to extend the color gamut and achieve good color quality in a reasonable range of colors. (See [the wiki page "Colors"](https://github.com/lenaschimmel/bestelampe/wiki/Colors) for the complexities involved)
 - **Smart, but not smart-ass** • Offers a wide range of wireless connection option, but is also a good, usable device when it's only connection is to the power grid.
 - **Cost efficient** • Should be less expensive than off-the-shelf smart lamps, even if they lack most of its features.
 - **Versatile** • Can be used for many different applications.
 - **Modular** • Provide many features, but not absurdly many, on the base module.
 - **Mains operated, but low voltage DC** • Don't mess with deadly AC.
 - **Power efficiency** • mostly by dimming and switching off.
 - **React to motion and presence** • IR-sensors can only detect motion, so a microwave- or mmwave-sensor must be supported.
 - **Hackable** • Over the air update, flash software and view logs via USB or UART. Pins for future expansions.
 - **Insect friendly** • Reduce the amount of light at night and offer insect friendly spectrum options.
 - **Sleep cycle aware** • Dim lights and avoid blue wavelength at night.
 - **Durable** • Should be usable for decades. Possibility to repair or replace individual parts or modules. Monitor it's own state and send warnings.
 - **Reliable** • Handle outages gracefully, no matter if power outage, network problems or minor component defects.
 - **Fun** • When all the important features are done, add some nice extras. Blinkenlighs-like light shows? UV light? Light based games? Let's do it!

## Not-to-do-list
 - Battery charging, photovoltaic, etc.
 - Addressable LEDs. There is a pin called "Neopixel" but that's not really a design priority, since most addressable LEDs lack in light quality (dimming steps, CRI, flicker)

## License
The besteLampe! is open hardware: 

- The hardware design is licensed under [CERN-OHL-S v2](https://ohwr.org/cern_ohl_s_v2.txt) or any later version, with one exception:
  - The ESP32-C6 Feather Module is licensed under 
- The software is licensed under [GNU General Public Licence v3.0](https://creativecommons.org/licenses/by-sa/4.0/legalcode) or later
- The documentation is licensed under [Creative Commons Attribution Share Alike 4.0 International](https://creativecommons.org/licenses/by-sa/4.0/)

<img src="assets/oshw_facts.svg" width="20%" />

See the [license](./license) directory for more details.