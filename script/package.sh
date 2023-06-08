#!/bin/sh
# Package the disk image into a virtual appliance using VirtualBox

set -e
. "$(dirname "$(which "$0")")/etc.sh"

MACHINE_NAME="$APP_NAME-$APP_VERSION-package-$( date --utc +%Y%m%d-%H%M%S )"
EXPORT_FILENAME="$APP_NAME-v$APP_VERSION-$( date --utc +%Y%m%d-%H%M%S ).ova"

echo "Creating virtual machine"

VBoxManage createvm --name $MACHINE_NAME --ostype Linux_64 --register

VBoxManage modifyvm $MACHINE_NAME \
    --memory 512 \
    --vram 16 \
    --acpi on \
    --ioapic on \
    --cpus 1 \
    --rtcuseutc on \
    --cpuhotplug off \
    --pae on \
    --hwvirtex on \
    --nestedpaging on \
    --largepages off \
    --accelerate3d off \
    --nic1 nat \
    --nictype1 82540EM \
    --natpf1 "Web interface,tcp,127.0.0.1,8001,,8001" \
    --audio-enabled off \
    --audio-driver none \
    --clipboard disabled \
    --usb off \
    --usbehci off \
    --mouse ps2 \
    --keyboard ps2 \
    --biosbootmenu menuonly

echo "Setting up storage"
VBoxManage storagectl $MACHINE_NAME \
  --name "SATA Controller" \
  --add sata \
  --portcount 4

echo "Compacting disk image"
VBoxManage modifymedium --compact "$OUTPUT_DIR/$VDI_DISK_FILENAME"

echo "Attaching disk image"
VBoxManage storageattach $MACHINE_NAME \
  --storagectl "SATA Controller" \
  --port 0 \
  --device 0 \
  --type hdd \
  --nonrotational on \
  --medium "$OUTPUT_DIR/$VDI_DISK_FILENAME"

echo "Exporting appliance"

VBoxManage modifyvm $MACHINE_NAME \
    --bioslogodisplaytime 0 \
    --bioslogofadein off \
    --bioslogofadeout off \
    --boot1 disk \
    --boot2 none \
    --boot3 none \
    --boot4 none \
    --biosbootmenu disabled

VBoxManage export $MACHINE_NAME \
    --output "$OUTPUT_DIR/$EXPORT_FILENAME" \
    --vsys 0 \
    --ovf20 \
    --manifest \
    --product "ArchiveTeam Warrior" \
    --vendor "ArchiveTeam" \
    --vendorurl "http://www.archiveteam.org/" \
    --version "$APP_VERSION" \
    --vmname "$APP_NAME-$APP_VERSION"

echo "Detaching disk image"
VBoxManage storageattach $MACHINE_NAME \
  --storagectl "SATA Controller" \
  --port 0 \
  --device 0 \
  --medium none

echo "Unregistering disk image"
VBoxManage closemedium "$OUTPUT_DIR/$VDI_DISK_FILENAME"

echo "Cleaning up by unregister and deleting virtual machine"
VBoxManage unregistervm $MACHINE_NAME --delete

echo "Package done"
