# RCPU\_OS

Kernel implementation running [RCPU][rcpu] based on [Writing an OS in
Rust][rust-os-blog]

## Implementation notes

- All numbers are unsigned 16 bit (u16)
	- Integer overflows can occur
		- For Add, Subtract, Multiply and Divide, the number wraps
		  around (mod 2^16)
		- For Left and Right Shifts, the number gets padded with zeroes
- The program expects a 16bit addressable memory space with the program loaded
  starting at 0
- The binary contains pre-allocated space, which will be loaded with the program
- All strings are ASCII (7 bit), not Latin-1
- RCPU\_OS stack grows **upwards** instead of downwards. RCPU has no way of
  reading from/writing to SP, so this should not matter to the programs.

[rcpu]: https://github.com/redfast00/RCPU
[rust-os-blog]: https://os.phil-opp.com/
