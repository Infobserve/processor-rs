# The infobserve processor

## Installation
Because we're lazy, there's no dockerfile yet
Here's how to install locally:

### Install Yara
For the processor to run, you'll need to have [Yara](https://github.com/VirusTotal/yara) installed

Dependencies

Ubuntu

```
# apt install automake libtool make gcc
```



Archlinux

(you also need to install `autoconf`)

```
pacman -S automake autoconf libtool make gcc
```

Now actually install yara:
```
~/ $ wget https://github.com/VirusTotal/yara/archive/v3.7.0.tar.gz
~/ $ tar zxf v3.7.0.tar.gz
~/ $ cd yara-3.7.0
~/yara-3.7.0 $ ./bootstrap.sh && ./configure && make && sudo make install
```

If you also want to make sure that Yara was installed correctly, go ahead and run
```
~/yara-3.7.0 $ make check
```


### Cargo
(Install rustup, cargo etc.)

```
$ git clone https://gitub.com/Infobserve/processor-rs
$ cd processor-rs
$ cargo install --path .
```

If the last command fails with something like the following: 
```
target/debug/processor-rs: error while loading shared libraries: libyara.so.3: cannot open shared object file: No such file or directory
```



(which is a problem with Yara linking), you might also need to run:
```
# ln -s /usr/local/lib/libyara.so /usr/lib/libyara.so.3
```



For the yara crate to compile, the `llvm-config` and `yara` executables must be available (they should be installed from the previous steps)

That's it, you *should* be good to go.

To run our wonderful and extensive test suite
```
$ cargo test
```
