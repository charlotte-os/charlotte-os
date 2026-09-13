build-catten arch="x86_64" profile="debug" features="":
    cargo build --package catten --target {{ if arch == "x86_64" { "target_specs/x86_64-unknown-none-catten.json" } else if arch == "aarch64" { "target_specs/aarch64-unknown-none-catten.json" } else if arch == "riscv64" { "target_specs/riscv64gc-unknown-none-catten.json" } else { arch + "-unknown-none" } }} {{ if profile == "release" { "--release" } else { "" } }} {{ if features !=
    "" {"--features " + features} else {""} }}

build-catten-docs arch="x86_64" profile="debug" features="":
    cargo doc --package catten --target {{ if arch == "x86_64" { "target_specs/x86_64-unknown-none-catten.json" } else if arch == "aarch64" { "target_specs/aarch64-unknown-none-catten.json" } else if arch == "riscv64" { "target_specs/riscv64gc-unknown-none-catten.json" } else { arch + "-unknown-none" } }} {{ if profile == "release" { "--release" } else { "" } }} {{ if features !=
    "" {"--features " + features} else {""} }} --no-deps --open

image_dir := "./os-images"
create-image arch="x86_64" profile="debug" features="": (build-catten arch profile features)
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{arch}}" in
        x86_64) bootloader="BOOTX64.EFI" ;;
        aarch64) bootloader="BOOTAA64.EFI" ;;
        riscv64) bootloader="BOOTRISCV64.EFI" ;;
        *) printf 'Unsupported image architecture: %s\n' "{{arch}}" >&2; exit 1 ;;
    esac
    image_path="{{image_dir}}/charlotte-{{arch}}-{{profile}}.img"
    mount_dir=""
    lodev=""
    cleanup() {
        local status=$?
        local cleanup_status=0
        trap - EXIT
        if [[ -n "$mount_dir" ]] && mountpoint -q "$mount_dir"; then
            if ! sudo umount "$mount_dir"; then
                # Leave the device attached if its filesystem is still mounted.
                if (( status == 0 )); then status=1; fi
                exit "$status"
            fi
        fi
        if [[ -n "$lodev" ]]; then
            sudo losetup -d "$lodev" || cleanup_status=1
        fi
        if [[ -n "$mount_dir" ]]; then
            rmdir "$mount_dir" || cleanup_status=1
        fi
        if (( status == 0 )); then status=$cleanup_status; fi
        exit "$status"
    }
    trap cleanup EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    mkdir -p "{{image_dir}}"
    # Reset existing data before creating a sparse, zero-filled 4 GiB image.
    truncate -s 0 "$image_path"
    truncate -s 4G "$image_path"
    parted -s "$image_path" mklabel gpt
    parted -s "$image_path" mkpart ESP fat32 1MiB 100%
    parted -s "$image_path" set 1 esp on
    lodev=$(sudo losetup -fP --show "$image_path")
    sudo mkfs.fat -F32 "${lodev}p1"
    mount_dir=$(mktemp -d)
    sudo mount "${lodev}p1" "$mount_dir"
    sudo mkdir -p "$mount_dir/EFI/BOOT"
    sudo cp "./limine-binary/$bootloader" "$mount_dir/EFI/BOOT/$bootloader"
    sudo cp "./target/{{ if arch == "x86_64" { "x86_64-unknown-none-catten" } else if arch == "aarch64" { "aarch64-unknown-none-catten" } else if arch == "riscv64" { "riscv64gc-unknown-none-catten" } else { arch + "-unknown-none" } }}/{{profile}}/catten" ./limine.conf "$mount_dir"

vm_memory := "512M"
vm_num_lps := "8"
usb_image_path := "./test_data/disk_images/test-usb.img"
# Padded, writable copies of the EDK2 riscv64 firmware; see qemu-run-riscv64.
riscv_fw_dir := image_dir / "riscv64-firmware"
# Where the guest's serial transcript is saved on the host; see the -chardev lines below.
log_dir := "./logs"

qemu-run-x86_64 profile="debug" features="qemu" iommu_type="amd" gdb="false": (create-image "x86_64" profile features)
    mkdir -p {{ log_dir }}
    qemu-system-x86_64 \
        -enable-kvm \
        -M q35,kernel-irqchip=split \
        -cpu host,+invtsc \
        -smp {{vm_num_lps}} \
        -m {{vm_memory}} \
        -drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2/ovmf/OVMF_CODE.fd \
        -boot d \
        -vga none \
        -device virtio-vga,xres=2560,yres=1440 \
        -drive file={{image_dir}}/charlotte-x86_64-{{profile}}.img,format=raw,if=none,id=nvme0 \
        -device nvme,drive=nvme0,serial=catten00 \
        -nic none \
        -device qemu-xhci,id=xhci \
        -device usb-kbd,bus=xhci.0 \
        -device usb-mouse,bus=xhci.0 \
        -netdev user,id=usbnet0 \
        -device usb-net,netdev=usbnet0,bus=xhci.0 \
        -device usb-storage,bus=xhci.0,drive=usbdrive0 \
        -drive if=none,id=usbdrive0,format=raw,file={{usb_image_path}} \
        {{ if iommu_type == "amd" {"-device amd-iommu,intremap=on,xtsup=on,dma-remap=on"} else {""} }} \
        {{ if iommu_type == "intel" {"-device intel-iommu,intremap=on,eim=on,dma-translation=on"} else {""} }} \
        -chardev stdio,id=catlog,signal=on,logfile={{ log_dir }}/catten-x86_64-{{ profile }}.log \
        -serial chardev:catlog \
        {{ if gdb == "true" {"-s -S"} else {""} }}

qemu-run-aarch64 profile="debug" gdb="false": (create-image "aarch64" profile)
    mkdir -p {{ log_dir }}
    qemu-system-aarch64 \
        -M virt,iommu=smmuv3 \
        -cpu cortex-a710 \
        -smp {{vm_num_lps}} \
        -m {{vm_memory}} \
        -bios /usr/share/edk2/aarch64/QEMU_EFI.fd \
        -boot d \
        -device ramfb \
        -device qemu-xhci,id=xhci \
        -device usb-kbd,bus=xhci.0 \
        -device usb-mouse,bus=xhci.0 \
        -netdev user,id=usbnet0 \
        -device usb-net,netdev=usbnet0,bus=xhci.0 \
        -drive file={{image_dir}}/charlotte-aarch64-{{profile}}.img,format=raw \
        -device usb-storage,bus=xhci.0,drive=usbdrive0 \
        -drive if=none,id=usbdrive0,format=raw,file={{usb_image_path}} \
        -chardev stdio,id=catlog,signal=on,logfile={{ log_dir }}/catten-aarch64-{{ profile }}.log \
        -serial chardev:catlog \
        {{ if gdb == "true" {"-s -S"} else {""} }}

qemu-run-riscv64 profile="debug" gdb="false": (create-image "riscv64" profile)
    #!/usr/bin/env bash
    set -euo pipefail
    # The riscv64 virt machine has two fixed 32 MiB flash slots, but EDK2 ships its firmware at
    # its natural size, so pad a copy of each to fit. The varstore also has to be writable, which
    # rules out pointing QEMU straight at the read-only files under /usr/share.
    mkdir -p {{riscv_fw_dir}}
    mkdir -p {{log_dir}}
    code="{{riscv_fw_dir}}/RISCV_VIRT_CODE.fd"
    vars="{{riscv_fw_dir}}/RISCV_VIRT_VARS.fd"
    for fw in CODE VARS; do
        dest="{{riscv_fw_dir}}/RISCV_VIRT_${fw}.fd"
        if [[ ! -f "$dest" ]]; then
            cp "/usr/share/edk2/riscv/RISCV_VIRT_${fw}.fd" "$dest"
            truncate -s 32M "$dest"
        fi
    done
    # riscv-iommu-pci defaults to off=on, which resets the IOMMU into Off mode and blocks all DMA
    # until software programs ddtp. Nothing here has a RISC-V IOMMU driver yet, so the xHCI
    # controller's DMA never lands and EDK2 spins forever on Enable Slot. off=off resets it to
    # Bare instead, leaving the device enumerable at 00:02.0 to write a driver against.
    qemu-system-riscv64 \
        -M virt \
        -cpu tt-ascalon \
        -smp {{vm_num_lps}} \
        -m {{vm_memory}} \
        -drive if=pflash,unit=0,format=raw,readonly=on,file="$code" \
        -drive if=pflash,unit=1,format=raw,file="$vars" \
        -boot d \
        -device ramfb \
        -device qemu-xhci,id=xhci \
        -device usb-kbd,bus=xhci.0 \
        -device usb-mouse,bus=xhci.0 \
        -netdev user,id=usbnet0 \
        -device usb-net,netdev=usbnet0,bus=xhci.0 \
        -device riscv-iommu-pci,off=off \
        -drive file={{image_dir}}/charlotte-riscv64-{{profile}}.img,format=raw \
        -device usb-storage,bus=xhci.0,drive=usbdrive0 \
        -drive if=none,id=usbdrive0,format=raw,file={{usb_image_path}} \
        -chardev stdio,id=catlog,signal=on,logfile={{log_dir}}/catten-riscv64-{{profile}}.log \
        -serial chardev:catlog \
        {{ if gdb == "true" {"-s -S"} else {""} }}

update-loc:
    tokei \
        --exclude target \
        --exclude .git \
        --exclude .vscode \
        --exclude .github \
        --exclude limine-binary \
        --exclude os-images \
    > loc.txt

clean:
    cargo clean
    rm -rf {{image_dir}}
    rm -rf {{log_dir}}

distclean: clean
    if [ -f Cargo.lock ]; then rm Cargo.lock; fi
