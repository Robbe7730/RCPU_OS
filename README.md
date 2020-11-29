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
- RCPU syscalls "return" by pushing the result to the stack
- RCPU `stream_num` is 0 for stdin, 1 for stdout
	- Reading from or writing to an invalid stream (everything except 0 and
	  1 respectively) causes a panic
- If Getc cannot read a character, it returns u16::MAX (aka -1 wrapped)

[rcpu]: https://github.com/redfast00/RCPU
[rust-os-blog]: https://os.phil-opp.com/
