# Developer information

## What's new

This appliance runs in the same manner as version 3. It uses Docker containers to host the warrior projects runner but with improvements.

* A custom Alpine Linux image is created instead of modifying a stock ISO.
* Two warrior services are installed: One that displays info to tty1 and another that manages the containers.
* The services still call shell scripts, but they have been modularized.
* dnscrypt-proxy is installed for encrypted DNS resolution

## Boot up process

Overview of what happens on boot up:

1. Service warrior4-appliance-display is started.
2. Service warrior4-appliance is started.
3. Containers (watchtower, watchtower run-once, warrior) are created if they do not exist.
4. Containers are updated using watchtower run-once.
5. Containers watchtower and warrior are started.
6. Wait for the warrior web interface to start up.
7. If steps 3 to 6 fail, they are retried or the system is rebooted.
8. Monitor the warrior container for reboot or poweroff.
9. If step 8 fails, they are retried or the system is rebooted.

## Building the appliance

Building the appliance is a two step process. Scripts are provided that does mostly everything automatically. A network connection is required as additional software needs to be downloaded.

* For scripts using Python, Python 3.11 or greater is required
* Builds require a Linux environment

### Common tasks for scripts using Docker

Create the build environment image:

```sh
python script/docker.py init
```

After building, unregister the image and clean up the Docker cache:

```sh
python script/docker.py remove
docker image prune
```

### Build disk image

Important: Building the disk image is recommended to be done in a virtual machine or container. The custom Alpine image script requires root access and may mess up your system.

### Using Docker

Attention: Using Docker to build is not supported and isn't working unless you run things with effectively root access.

The Alpine build scripts require a Linux NBD. Enable nbd on the Docker host:

```sh
sudo modprobe nbd
```

On an unused device, give permissions as needed. (In this command, we give everyone read/write access. Alternatively, you can change the owner/group of the device.):

```sh
sudo chmod o+rw /dev/nbd6
```

Build in the container with the device:

```sh
python script/docker.py build --device "/dev/nbd6"
```

### Manually

These steps have been tested on Ubuntu 22.04.

Install the dependencies:

```sh
apt install build-essential musl-tools qemu-utils curl wget rsync
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
python ./script/package.py
```

The ova file should be generated in the `output` directory. That file is the virtual appliance. You may delete the vdi file now.

## Logging into the virtual machine

Switch to tty3 console (using guest keyboard) using either Alt+RightArrow or Alt+F3.

## Creating patches

The `appliance/script/patch.sh` is downloaded by the VM from the git "patch" branch and executed to patch the system. It first backs up the existing executables, applies any system modifications, and installs an Alpine package (.apk) to install newer executables.

### Using Docker

Set up the Docker image, then run:

```sh
python script/docker.py apk
```

### Manually

Within a virtual machine, run:

```sh
script/apk.sh
```

### Testing the patch

To test the apk and patching process, you can configure the option `patch_script_url` within the `/etc/warrior4-appliance.toml` file of the virtual machine. You can start up a local web server on the host using something like `python3 -m http.server`. If you are using VirtualBox NAT, 10.0.2.2 is forwarded to your host's localhost interface.

Example config value:

```toml
patch_script_url = "http://10.0.2.2:8000/appliance/script/patch.sh"
```
