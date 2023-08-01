# stage0-bootloader
The purpose of the first stage is to be small enough to fit within the 512 byte MBR's (Master Boot Record) first sector.
During stage 0 the CPU is in 'Real' mode, which is 16-bit. The loader is loaded at address 0000:7c00, and it's main job
is to find the active partition that contains the rest of the bootloader.