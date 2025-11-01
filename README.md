# Mpkg

## About

Mpkg is an alternative to npm, it has its own registry and supports npm via a wrapper.

Mpkg can also be used in replacement for nodejs although this is also just a wrapper.

## Info

You can upload and view packages at mpkg.idiotstudios.co.za

Mpkg consists of a few things, the cli, registry and website.

The website it a work in progress as the server's backend it being rebuilt

## Installation

In every linux/macOS release, there is an install.sh, running this file as follows will install mpkg:

```bash
bash install.sh
```

On Windows you can run the msi file included which will install it systemwide and add it to path.

## Usage

Mpkg can only be used for ESM not CommonJs, but lets be honest, no one likes CommonJs anyway...

To install any package that is in the mpkg registry, run the following:
```bash
mpkg install <package-name>
```

To install any package that is in the npm registry, run the following:
```bash
mpkg install-npm <package-name>
```
Npm must be pre-installed as we have no implmented a proper npm system yet.

To initialize a directory you can run the following:
```bash
mpkg init <project-name>
```

To run any javascript file, use the follwing:
```bash
mpkg run <server.js>
```
Nodejs must be installed as we have no implmented our own version as of yet.

## Contribution

Feel free to contribute in anyway via issues and or pull requests.
