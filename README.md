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
  * In this case, you will need to download the vmdk (VMware) or qcow2 (QEMU) disk images and use that as the disk for a new virtual machine. Then, configure the machine as described in the next section.
* Some issues may still remain from version 3.

### Suggested defaults for manual configuration

If you are manually creating a new virtual machine using a disk image, here are the suggested defaults:

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
