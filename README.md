# Mpkg

## About

Mpkg is an alternative to npm, it has its own registry and supports npm via a wrapper.

Mpkg can also be used in replacement for nodejs although this is also just a wrapper.

## Info

Mpkg consists of a few things, the cli and the registry.

The website is a work in progress at the moment. (have not started)

The cli can be used as follows:

```bash
mpkg install <package-name> // Installs any package in the default mpkg registry (not hosted yet)

mpkg install-npm <package-name> // Installs any npm package via an npm wrapper

mpkg init <project-name> // will initialize the directory it is run in by adding a gitignore and a pkg.jsoc

mpkg run <server.js> // will run anything that nodejs would be able to run via a wrapper
```

The Registry can be used by api requests, but I am not going to go into that here

## Usage

Mpkg can only be used for ESM not CommonJs, but lets be honest, no one likes CommonJs anyway...

I basically covered this in Info so Im not going to go more into Usage

## Contribution

Feel free to contribute in anyway, this is just a side project of mine.
