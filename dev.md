# Developer information

## What's new

This appliance runs in the same manner as version 3. It uses Docker containers to host the warrior projects runner but with improvements.

* A custom Alpine Linux image is created instead of modifying a stock ISO.
* Two warrior services are installed: One that displays info to tty1 and another that manages the containers.
* The services still call shell scripts, but they have been modularized.
* dnscrypt-proxy is installed for encrypted DNS resolution

## Building the appliance

Building the appliance is a two step process. Scripts are provided that does mostly everything automatically. A network connection is required as additional software needs to be downloaded.

### Build disk image

Important: Building the disk image is recommended to be done in a virtual machine. The custom Alpine image script requires root access and may mess up your system.

These steps have been tested on Ubuntu 22.04.

Install the dependencies:

```sh
apt install build-essential musl-tools qemu-utils curl rsync
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add x86_64-unknown-linux-musl
```

These packages should be installed already on Ubuntu, but if they aren't, run:

```sh
apt install fdisk e2fsprogs
```

Once the depedencies are installed, run the script to build the image:

```sh
./script/build.sh
```

The generated image will be in the `output` directory. You'll need the vdi file for the next step. The qcow2 file is not needed but is not automatically deleted in case you need it. You should delete the qcow2 file if you rerun the build script.

### Package the virtual machine

Packaging the virtual machine requires VirtualBox for use of its utilities.

These steps have been tested using VirtualBox 7 installed on Ubuntu.

(You don't need to do this step in a virtual machine, but the script does temporarily register a machine. If you do run VirtualBox in a virtual machine, you'll need to enable nested virtualization for your hypervisor.)

(If you are setting up on a physical machine that is already running another virtual machine solution, note that running more than one hypervisor at a time may not be possible. You might get an error when you attempt to boot a VirtualBox machine.)

Ensure the vdi file from the previous is in the `output` directory.

Then run:

```sh
./script/package.sh
```

The ova file should be generated in the `output` directory. That file is the virtual appliance. You may delete the vdi file now.
