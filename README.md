# The infobserve processor

## Installation
Because we're lazy, there's no dockerfile yet
Here's how to install locally:

### Install Yara
The processor depends on [Yara](https://github.com/VirusTotal/yara) to be installed locally. No manual installation is required
as Yara is now included with the crate we use for exporting its bindings ([https://github.com/Hugal31/yara-rust/](yara-rust)).
If a manual installation is preferred, the [installation steps can be found in Yara's documentation](https://yara.readthedocs.io/en/stable/gettingstarted.html)

### Cargo
(First install rustup, cargo etc.)

```
$ git clone https://gitub.com/Infobserve/processor-rs
$ cd processor-rs
$ cargo check # Check that everything is OK
```

That's it, you *should* be good to go.

To run our wonderful and extensive test suite
```
$ cargo test
```
