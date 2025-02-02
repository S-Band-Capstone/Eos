# Eos
A tool developed to be used by educational organizations when configuring an S-Band radio that uses the CC2510/CC2511 chips by TI.

## Structure
The app is built using Rust and it's Tokio & Tokio-Serial crates for serialization communication, and egui for the UI.


## Use
By changing the values of specific registers, this application can adjust the following parameters for your radio:
 - Frequency
 - Bandwidth
 - Data Rate
 - Modulation Scheme
 - Data Whitening
 - Manchester Enable
 - Transmit power
 - Phase Transition Time
 - Deviation
 - Channel Bandwidth

