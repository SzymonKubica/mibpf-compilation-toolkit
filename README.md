# mibpf-tools

Compilation and deployment toolkit for [mibpf](https://github.com/SzymonKubica/mibpf) (μBPF)

This repository contains a set of tools, scripts and definitions that
can be used to develop applications for microcontrollers using eBPF.

It is meant to work in conjunction with [μBPF](https://github.com/SzymonKubica/mibpf) - a port of an eBPF VM for microcontrollers running [RIOT](https://www.riot-os.org/).

**mibpf-tools** consist of three separate packages:
- `tools`
- `common`
- `elf-utils`

### `tools`

This module contains a CLI application which allows for compiling, verifying,
deploying and executing eBPF programs on microcontrollers running an instance of
the μBPF VM.

### `common`

This module contains definitions of constants, structs and enums that are shared
between the CLI tool and the server application that is responsible for executing
instances of the VM on the target microcontrollers.

### `elf-utils`

This module contains a set of functions responsible for modifying ELF files that
result from compiling eBPF programs. It is used by both the CLI tool and the
main application server to transform program into custom binary formats that
are specific to the implementation of the VM.

It can also act as a static linker to allow for resolving relocations in the ELF
files directly on the target device.

