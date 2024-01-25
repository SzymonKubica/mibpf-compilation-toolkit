
# Need to replace the instructions as explained here: https://github.com/qmonnet/rbpf/blob/main/examples/load_elf.rs
# This feels sketchy but apparently the linux kernel does that as well.
# The purpose of this manipulation is to replace the instructions responsible
# for loading the packet data as 4-byte words for instructions that will load
# 8-byte double words (instructions staring with 0x79).
# Further, we need to change the offset at which the pointer to the packet
# data is stored as the values produced by clang would cause the addresses to
# overlap
#
# TODO: make this work for all programs
xxd $1 | tee checkpoint | sed ' s/6112 5000 0000 0000/7912 5000 0000 0000/ ;
    s/6111 4c00 0000 0000/7911 4000 0000 0000/ ;
   s/6111 2200 0000 0000/7911 2200 0000 0000/' | xxd -r > $1.tmp

mv $1.tmp $1

