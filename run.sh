rm -rf ./.root

if ! test -d "kernel/lai"; then
    git clone https://github.com/managarm/lai.git --depth 1
fi

if ! test -d "vmem"; then
    git clone git@github.com:archaic-archea/vmem.git --depth 1
    cd vmem
    cargo build
    cd ..
fi

cd kernel
zig build

if [ $? -eq 0 ]; then
    echo Build success
else
    echo Build failure
    exit
fi

cd ..

if [ $(uname) == "Darwin" ]; then
    truncate -s 33554432 EDK2.fd 
else 
    truncate CODE.fd --size 33554432
fi

if ! test -d "limine"; then
    echo Grabbing Limine
    git clone https://github.com/limine-bootloader/limine.git --depth 1 --branch=v5.x-branch-binary 
fi 
make -C limine
mkdir -p .root
#cp -v kernel/target/riscv64imac-unknown-none-elf/release/gent-kern config/limine.cfg limine/limine-bios.sys \
cp -v zig-out/bin/GeNT config/limine.cfg limine/limine-bios.sys \
      limine/limine-bios-cd.bin limine/limine-uefi-cd.bin \
      .root/
mkdir -p .root/EFI/BOOT
cp -v limine/BOOTRISCV64.EFI .root/EFI/BOOT/

qemu-system-riscv64-acpi \
    -machine virt,aclint=on,acpi=on,aia=aplic-imsic \
    -cpu rv64,svpbmt=on \
    -smp 4 \
    -m 4G \
    -pflash CODE.fd \
    -device nvme,serial=deadbeff,drive=disk1 \
    -drive id=disk1,format=raw,if=none,file=fat:rw:./.root \
    -global virtio-mmio.force-legacy=false \
    -device ramfb \
    -serial mon:stdio \
    -d int \
    -D debug.log