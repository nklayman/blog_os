from bootloader.build import main
import os
os.system("cargo build")
os.chdir("./bootloader")
main()
