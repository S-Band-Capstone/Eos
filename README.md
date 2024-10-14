# Eos
A tool developed to be used by educational organizations when configuring an S-Band radio that uses the CC2510/CC2511 chips by TI.

## Structure
The app is built using [Tauri](https://v2.tauri.app/), a framework that containerizes development of an application to make it cross-platform.

### Frontend
This application's frontend is developed using Vue and TypeScript.

### Backend
All Tauri applications use Rust for their backend and window management.


## Use
By changing the values of specific registers, this application can adjust the following parameters for your radio:
 - Frequency
 - Bandwidth
 - Data Rate
 - Modulation Scheme

