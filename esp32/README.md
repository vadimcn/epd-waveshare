
# Hardware

This example was created for [Waveshare E-Paper ESP32 Driver Board](https://www.waveshare.com/wiki/E-Paper_ESP32_Driver_Board) 
installed in [12.48inch E-Paper Module](https://www.waveshare.com/wiki/12.48inch_e-Paper_Module_(B)).

# Initial setup

1. Follow instructions from [The Rust on ESP Book](https://esp-rs.github.io/book/installation/index.html).
2. Update `PATH` and `LIBCLANG_PATH` in `.cargo/config.toml` to point to the toolchain installation directory, using
values from `$HOME/export-esp.sh` that `espup` [creates](https://esp-rs.github.io/book/installation/riscv-and-xtensa.html).
Setting up via `. $HOME/export-esp.sh` is also possible, but not recommended for IDE users, because it's hard to ensure 
that Rust tools spawned by the IDE (such as cargo and rust-analyzer) would inherit the correct environment.
