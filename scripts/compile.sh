clang -O2 -emit-llvm -c $1.c -o - | llc -march=bpf -filetype=obj -o $1.o
