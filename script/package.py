#!/bin/env python3
# Package the disk image into a virtual appliance using VirtualBox,
# compress QCOW2 and VMDK archives
import datetime
import gzip
import os
import shutil
import subprocess
import tomllib
import zipfile

with open(os.path.join(os.path.dirname(__file__), "etc.sh"), "rb") as f:
    config = tomllib.load(f)

APP_NAME = config["APP_NAME"]
APP_VERSION = config["APP_VERSION"]
OUTPUT_DIR = config["OUTPUT_DIR"]
QCOW2_DISK_FILENAME = config["QCOW2_DISK_FILENAME"]
VDI_DISK_FILENAME = config["VDI_DISK_FILENAME"]
VMDK_DISK_FILENAME = config["VMDK_DISK_FILENAME"]
VHDX_DISK_FILENAME = config["VHDX_DISK_FILENAME"]
DATE = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%d-%H%M%S")
MACHINE_NAME = f"{APP_NAME}-{APP_VERSION}-package-{DATE}"
BASENAME = f"{APP_NAME}-v{APP_VERSION}-{DATE}"
EXPORT_FILENAME = f"{BASENAME}.ova"


def vbox(args):
    subprocess.run(
        [
            "VBoxManage",
        ]
        + args,
        check=True,
    )


def package_virtualbox():
    print("Creating virtual machine")
    vbox(
        [
            "createvm",
            "--name",
            MACHINE_NAME,
            "--ostype",
            "Linux_64",
            "--register",
        ]
    )
    vbox(
        [
            "modifyvm",
            MACHINE_NAME,
            "--memory",
            "512",
            "--vram",
            "16",
            "--acpi",
            "on",
            "--ioapic",
            "on",
            "--cpus",
            "1",
            "--rtcuseutc",
            "on",
            "--cpuhotplug",
            "off",
            "--pae",
            "on",
            "--hwvirtex",
            "on",
            "--nestedpaging",
            "on",
            "--largepages",
            "off",
            "--accelerate3d",
            "off",
            "--nic1",
            "nat",
            "--nictype1",
            "82540EM",
            "--natpf1",
            "Web interface,tcp,127.0.0.1,8001,,8001",
            "--audio-enabled",
            "off",
            "--audio-driver",
            "none",
            "--clipboard",
            "disabled",
            "--usb",
            "off",
            "--usbehci",
            "off",
            "--mouse",
            "ps2",
            "--keyboard",
            "ps2",
            "--biosbootmenu",
            "menuonly",
        ]
    )
    print("Setting up storage")
    vbox(
        [
            "storagectl",
            MACHINE_NAME,
            "--name",
            "SATA Controller",
            "--add",
            "sata",
            "--portcount",
            "4",
        ]
    )

    print("Compacting disk image")
    vbox(["modifymedium", "--compact", f"{OUTPUT_DIR}/{VDI_DISK_FILENAME}"])

    print("Attaching disk image")
    vbox(
        [
            "storageattach",
            MACHINE_NAME,
            "--storagectl",
            "SATA Controller",
            "--port",
            "0",
            "--device",
            "0",
            "--type",
            "hdd",
            "--nonrotational",
            "on",
            "--medium",
            f"{OUTPUT_DIR}/{VDI_DISK_FILENAME}",
        ]
    )

    print("Exporting appliance")

    vbox(
        [
            "modifyvm",
            MACHINE_NAME,
            "--bioslogodisplaytime",
            "0",
            "--bioslogofadein",
            "off",
            "--bioslogofadeout",
            "off",
            "--boot1",
            "disk",
            "--boot2",
            "none",
            "--boot3",
            "none",
            "--boot4",
            "none",
            "--biosbootmenu",
            "disabled",
        ]
    )

    vbox(
        [
            "export",
            MACHINE_NAME,
            "--output",
            f"{OUTPUT_DIR}/{EXPORT_FILENAME}",
            "--vsys",
            "0",
            "--ovf20",
            "--manifest",
            "--product",
            "ArchiveTeam Warrior",
            "--vendor",
            "ArchiveTeam",
            "--vendorurl",
            "http://www.archiveteam.org/",
            "--version",
            APP_VERSION,
            "--vmname",
            f"{APP_NAME}-{APP_VERSION}",
        ]
    )

    print("Detaching disk image")
    vbox(
        [
            "storageattach",
            MACHINE_NAME,
            "--storagectl",
            "SATA Controller",
            "--port",
            "0",
            "--device",
            "0",
            "--medium",
            "none",
        ]
    )

    print("Unregistering disk image")
    vbox(["closemedium", f"{OUTPUT_DIR}/{VDI_DISK_FILENAME}"])

    print("Cleaning up by unregister and deleting virtual machine")
    vbox(["unregistervm", MACHINE_NAME, "--delete"])


def package_qcow2():
    print("Creating QCOW2 archive")

    with gzip.GzipFile(
        filename=f"{OUTPUT_DIR}/{BASENAME}.qcow2.gz", mode="wb", compresslevel=9
    ) as dest, open(f"{OUTPUT_DIR}/{QCOW2_DISK_FILENAME}", "rb") as source:
        shutil.copyfileobj(source, dest)


def package_vmdk():
    print("Creating VMDK archive")

    with zipfile.ZipFile(
        f"{OUTPUT_DIR}/{BASENAME}.vmdk.zip",
        mode="w",
        compression=zipfile.ZIP_DEFLATED,
        compresslevel=9,
    ) as zip:
        name = f"{BASENAME}.vmdk"
        zip.write(f"{OUTPUT_DIR}/{VMDK_DISK_FILENAME}", name)


def package_vhdx():
    print("Creating VHDX archive")

    with zipfile.ZipFile(
        f"{OUTPUT_DIR}/{BASENAME}.vhdx.zip",
        mode="w",
        compression=zipfile.ZIP_DEFLATED,
        compresslevel=9,
    ) as zip:
        name = f"{BASENAME}.vhdx"
        zip.write(f"{OUTPUT_DIR}/{VHDX_DISK_FILENAME}", name)


package_virtualbox()
package_qcow2()
package_vmdk()
package_vhdx()


print("Package done")
