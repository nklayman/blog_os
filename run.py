from bootloader.build import run_qemu
import os
os.system("cargo build")
os.chdir("./bootloader")
run_qemu()
