# Chip-8 Emulator

A project to teach myself the basics of emulation.
Following [this introduction](https://github.com/aquova/chip8-book?tab=readme-ov-file) and [this technical ref](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.3)


## Build
If you don't want to build this project from source, it is also available in your browser over [here](https://omathot.com/emulators)\
If you do:
### Requirements
- rust toolchain
- SDL3

```
git clone https://github.com/omathot/chip8.git
cd chip8
cargo run --manifest-path desktop/Cargo.toml -- <path to rom>
```

## Input Mapping
```
Keyboard:  ->  Chip-8:
|1|2|3|4|      |1|2|3|C|
|q|w|e|r|      |4|5|6|D|
|a|s|d|f|      |7|8|9|E|
|z|x|c|v|      |A|0|B|F|
```

## Screenshots
![image](https://github.com/user-attachments/assets/0a7344fb-4e0b-4916-bd9f-d861a7fa5a25)\
![image](https://github.com/user-attachments/assets/2c66af42-866c-47f6-90f3-72d54a193f16)\
![image](https://github.com/user-attachments/assets/63aae47c-c437-44dd-9297-c51fe2346cab)
