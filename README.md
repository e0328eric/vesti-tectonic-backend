# vesti-tectonic-git

I don't know why that the tectonic stable backend does not compiled. But at
least, for recent tectonic branch it does compile.

This repo is purposed to compile vesti with tectonic backend. I tested it on
MSYS2 sandbox on Windows 11.

To compile this, run the following command:
```console
$ cargo b --release --features=tectonic-backend
```

If one does not want to use tectonic backend but using local LaTeX compiler to
compile vesti, see [the main vesti repo](https://github.com/e0328eric/vesti).
