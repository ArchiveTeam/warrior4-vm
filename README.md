# Warrior 4 VM

This repository contains the [Warrior](https://wiki.archiveteam.org/index.php/ArchiveTeam_Warrior) Virtual Machine Appliance Version 4 for [ArchiveTeam](https://archiveteam.org).

*Prereleases are available for testing. Note that they are work-in-progress and may not be stable yet.*

💿 If you are looking to download the warrior, a ready-to-use virtual appliance is located at the Releases section or at [Warrior HQ downloads](https://warriorhq.archiveteam.org/downloads/warrior4/). You'll need a virtualization solution of your choice such as [VirtualBox](https://www.virtualbox.org/).

🐋 If you are looking to run the warrior in a Docker container, use the image available from [warrior-dockerfile](https://github.com/ArchiveTeam/warrior-dockerfile).

If you want to see the older version 3, see [Ubuntu-Warrior](https://github.com/ArchiveTeam/Ubuntu-Warrior).

## Support

* If you need help or troubleshooting information, check the [FAQs](https://wiki.archiveteam.org/index.php/ArchiveTeam_Warrior) or Issues section to see if your problem has been answered already. If you want to to discuss in general, join the warrior IRC channel.
* For bug reports, please file an issue on the Issues section.

### Known issues

* OVA file may not be fully compatible with other virtual machine software.
  * In this case, you will need to download the disk image instead and manually configure a new virtual machine to use them. See the next section for details.
* Some issues may still remain from version 3.

### Using the disk images

Using the disk image directly is intended for advanced cases or when the OVA file is not compatible. This will expand to 60GB.

In some cases, such as using ESXi, you may need to "reformat" the image to work. This can be done using `vmkfstools -i input.vmdk output.vmdk`. See [VMWare docs](https://kb.vmware.com/s/article/1028943) for more information.

To use the disk image:

1. Download the supported format: vmdk (VMware) or qcow2 (QEMU).
2. Decompress them (unzip/gunzip).
3. Move the disk image file to where your virtual machines are saved. Since the VM software will use this existing file as its storage disk, you may want to keep a copy of the original to avoid having to redownload a new disk image.
4. Create a new machine (select either Alpine Linux 64-bit variant if available, or else "Other Linux (64-bit)") and choose the existing disk file that you have downloaded and moved.
5. Configure the machine as described in the next section.

Here are the suggested defaults:

| Hardware | Value |
| -------- | ----- |
| Memory | 512 MB |
| Video memory | 16 MB |
| 3D graphics | off |
| CPUs | 1 |
| Network | NAT |
| USB | off |
| Audio | off |

When using NAT network type, port forwarding is required to access the web interface. The following allows you to access port 8001 on localhost that is forwarded to the VM's port 8001:

| Name | Host address | Host port | VM address | VM port |
| ---- | ------------ | --------- | ------------- | ---------- |
| Web Interface | 127.0.0.1 | 8001 | | 8001 |

## Developer

Please see [dev.md](dev.md) for instructions on how to build the appliance and more information about the appliance.
