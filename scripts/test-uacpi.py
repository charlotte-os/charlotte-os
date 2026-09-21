#!/usr/bin/env python3
"""Boot the uACPI host tests and inject two real power-button SCIs per VM.

Uses temporary FAT images and QEMU TCG; no mounts, KVM, or root privileges needed.
"""
import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
READY = "[ACPI self-test] Ready for power-button SCI"
PASSED = "[ACPI self-test] Power-button SCI and deferred work passed"


def run(*args):
    subprocess.run(args, cwd=ROOT, check=True)


def smoke(image, firmware, machine, cpus, log):
    log.write_text("")
    command = [
        "qemu-system-x86_64", "-accel", "tcg", "-M", machine,
        "-cpu", "max,phys-bits=36,la57=off", "-smp", str(cpus), "-m", "512M",
        "-drive", f"if=pflash,format=raw,readonly=on,file={firmware}",
        "-drive", f"format=raw,file={image}", "-nic", "none",
        "-display", "none", "-serial", f"file:{log}", "-monitor", "stdio", "-no-reboot",
    ]
    with log.with_suffix(".monitor.log").open("wb") as monitor, \
            subprocess.Popen(command, stdin=subprocess.PIPE, stdout=monitor) as vm:
        try:
            deadline = time.monotonic() + 90
            sent = 0
            while time.monotonic() < deadline:
                output = log.read_text(errors="replace") if log.exists() else ""
                if "kernel panic" in output.lower() or vm.poll() is not None:
                    raise RuntimeError(f"{machine}/{cpus} CPUs failed; see {log}\n{output[-6000:]}")
                completed = output.count(PASSED)
                if completed == 2 and "Platform initialization complete." in output:
                    print(f"PASS: {machine}, {cpus} CPUs; host tests + two SCIs ({log})", flush=True)
                    return
                if READY in output and sent < 2 and completed == sent:
                    vm.stdin.write(b"system_powerdown\n")
                    vm.stdin.flush()
                    sent += 1
                time.sleep(0.05)
            raise RuntimeError(f"{machine}/{cpus} CPUs timed out; see {log}\n{output[-6000:]}")
        finally:
            if vm.poll() is None:
                try:
                    vm.stdin.write(b"info registers\nquit\n")
                    vm.stdin.flush()
                    vm.wait(timeout=5)
                except (BrokenPipeError, subprocess.TimeoutExpired):
                    vm.kill()
                    vm.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--firmware", type=Path, default=Path("/usr/share/edk2/ovmf/OVMF_CODE.fd"))
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument("--cpus", nargs="+", type=int, default=[1, 2, 8])
    args = parser.parse_args()
    if not args.firmware.is_file():
        parser.error("UEFI firmware not found; pass --firmware /path/to/OVMF_CODE.fd")
    if any(cpus < 1 for cpus in args.cpus):
        parser.error("CPU counts must be positive")
    for tool in ["qemu-system-x86_64", "mformat", "mmd", "mcopy"]:
        if shutil.which(tool) is None:
            parser.error(f"missing tool: {tool}")
    if not args.no_build:
        run("cargo", "build", "--package", "catten", "--target",
            "target_specs/x86_64-unknown-none-catten.json", "--features", "acpi_self_test")
    (ROOT / "logs").mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="catten-uacpi-") as temp:
        image = Path(temp) / "esp.img"
        with image.open("wb") as stream:
            stream.truncate(128 * 1024 * 1024)
        run("mformat", "-i", str(image), "-F", "::")
        run("mmd", "-i", str(image), "::/EFI", "::/EFI/BOOT")
        run("mcopy", "-i", str(image), "limine-binary/BOOTX64.EFI", "::/EFI/BOOT/BOOTX64.EFI")
        run("mcopy", "-i", str(image), "target/x86_64-unknown-none-catten/debug/catten", "limine.conf", "::/")
        for cpus in args.cpus:
            smoke(image, args.firmware.resolve(), "q35", cpus,
                  ROOT / "logs" / f"uacpi-smoke-q35-{cpus}.log")


if __name__ == "__main__":
    main()
