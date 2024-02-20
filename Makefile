build:
	make -C hvisor build
disasm:
	make -C hvisor disasm
debug:
	make -C hvisor debug
monitor:
	make -C hvisor monitor
clean:
	make -C hvisor clean
run:
	make -C hvisor run
dtb:
	make -C hvisor dtb

.PHONY: build disasm debug monitor clean run dtb