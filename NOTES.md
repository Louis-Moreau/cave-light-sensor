
```
sudo service udev start
sudo udevadm control --reload-rules
sudo service udev restart
```

rustup toolchain install nightly
rustup target add --toolchain nightly thumbv6m-none-eabi
cargo +nightly build
