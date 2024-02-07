if  [ $# -lt 2 ]; then
    echo "Usage: $0 <bpf-bin-file> <coaproot-dir>"
    exit 1
fi

bin_file=$1
coaproot_dir=$2
make -C bpf/femto-container clean
echo "Compiling the eBPF binary."
make -C bpf/femto-container all
echo "Copying the binary: $bin_file to the coap root directory"
cp bpf/femto-container/*.o bpf/femto-container/out

# ensure the build directory exists
[ -e build ] || mkdir build
cp bpf/femto-container/$bin_file build
