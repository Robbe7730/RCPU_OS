echo """
set timeout=0
set default=0

menuentry "my os" {
    multiboot2 /boot/kernel.bin
"""
for file in $1/*.out; do
	filename=`basename $file`
	echo "    module2 /boot/$filename `echo \"$filename\" | sed s/.out$//`"
done
echo """
    boot
}
"""
