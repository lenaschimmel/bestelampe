<!-- *Apart from this README, most documentation and notes are in German. If this project becomes more than a proof-of-concept, all important parts the documentation will be translated to English. The name will remain German, as it should be understandable to English speakers as well.* -->

![Header image with PCB closeup](assets/header.jpg)

## besteLampe!
**Most lamps suck. This is an open source hardware and software project to create not only a better lamp, but the best lamp. Or *"besteLampe!"* as we say in German.**

Of course, there are **many** kinds of lamps, and for some use cases, better alternatives exist. This one is created with two main use cases in mind:
 - indoor lamp for general room lighting and as a sunrise alarm clock 
 - outdoor lamp in front of apartments

The first case involves more direct, manual control of a single lamp, and the second case is more about automatic control (time based, responding to movement, linked with nearby lamps). Anyway, the lamp in each use case will sometimes be operated *just like in the other use case*, so both have mostly the same requirements, just with different priorities.

## What makes ~~a good~~ the best lamp?
These are the design goals of *besteLampe!* 

 - **Open** •  open hardware, open software, open protocols *(Attention: see at the very bottom)*
 - **State-of-the-art dimming** • Flicker free, smooth dimming, extremely wide contrast ratio (a.k.a "dim to zero"), independent dimming of color and brightness.
 - **Premium color control** • Use as many color channels as needed to extend the color gamut and achieve good color quality in a reasonable range of colors.
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

*Note: The current version **strives** to achieve all of them, but I can't (yet) say whether it actually accomplishes them all.*

## Not-to-do-list
 - Battery charging, photovoltaic, etc.
 - Addressable LEDs. There is a pin called "Neopixel" but that's not really a design priority, since most addressable LEDs lack in light quality (dimming steps, CRI, flicker)

## Parts
The besteLampe! project has multiple parts. In the future, they might be moved into separate repositories. They are:

 - Hardware
   - [x] **[Main module](hardware/mainmodule/)** • Control board with power supply, micro controller, 6-channel PWM (pulse width modulation) dimmer and many ports. *First version shipped and tested*
   - [ ] **Extension module** • Additional simultaneous wireless protocols, 16 additional PWM channels.
   - [ ] **LED modules** • Stuff that actually emits light. *only handmade prototypes*
   - [-] **Enclosure** • To hold the electronics together, protect it from the environment, and guide the light.
 - Software
   - [-] **[Firmware]() for main- and extension module**
   - [-] **Control software (Web, Desktop, Mobile, CLI, M5Paper...)**
   - [-] **Common code library** • The firmware and all control software variants are written in Rust and use a common code base.

## State of the project
### Software
The actual software is basically non-existent. There are a few proof-of-concept projects written in Rust, but they don't have the proper architecture. Most of these projects are not even worth putting into git.

A concept for a reliable, flexible firmware exists in my head, but I did not yet have time to write it down.

Possible next steps:
 - Fix multithreaded access to peripherals
 - Move code between bestelampe (hardware-dependent) and abstraktelampe (hardware-independent)
 - Check if fading and temporal dithering are already available in Rust

### Hardware
For the hardware, multiple prototypes with various degrees of sophistication exist:

<img src="assets/first_prototype.jpg" width="49%" /><img src="assets/second_prototype.jpg" width="49%" />
<img src="assets/pcb_assembled.jpg" width="98%" />

I already ordered PCBs from JLCPCB for the Main Module v1, and even though they have a lot of minor bugs, they look great and - even more important - are fully functional apart from the missing light sensor. **I do not recommend to order Main Module v1.** There are a lot of really easy-to-fix nuisances, so please wait for version 1.1 or version 2. **If you really want a v1 board in your hands quickly, just contact me. I have a few spare ones (partly assembled).**

I'm working on multiple enclosure designs, one based on a transparent flower pot, and one based on an existing lamp where I replace all the electronics with my own. Both contain many 3d-printed parts,  and a lot of manual labour to make them look *less 3d-printed*.

The next steps for the hardware could be:
 - Order light the sensor and test it on v1
 - Fix bugs in Main Module v1 and release v1.1
 - Find high quality SMD LEDs that are in stock at JLCPCB, PcbWay or some similar manufacturer
 - Design the first LED module and order them
 - Put a second ESP32 into the prototype to test ESP-AT or similar communication between both ESPs

 ## License
 This project *should* really be open. I have not yet decided on a license, bit feel like that should longer stop me from publishing it. So technical, this is not yet open hardware / free software. I'll fix that soon!
