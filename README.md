# LrnProf

## Current state and context

LrnProf is a profiler fully written in Rust that currently targets only Mach-O binary files. It is written from scratch using hand parsing with structures obtained from Apple documentation.

I use several repositories explaining how a binary file is structured and the different tables in it. I also use Chat-GPT to explain many concepts that I didn't understand when I started this project.

Currently, the profiler is not fully functional. While some features are operational, the main goal of tracing the call stack and unwinding it for debugging purposes is not yet working. At first, I was parsing the binary file directly. From the PID, I was able to locate and parse the binary file directly to retrieve data inside it. But you cannot correctly retrace the call stack using this method, and that's not even the purpose of a profiler. So I restarted almost from the beginning.

Currently, the profiler can parse a target process from its PID and parse it directly from its address space. It can parse the Mach-O header, define what type of Mach-O it is (currently only 32 and 64 bits) using the magic number in little-endian.

After that, it can parse the load commands and store them in a vector. But I can only get 17 out of 18 (from ncmds) from the target process, and I don't know why. Then I can retrieve the symtab and some lc_segment_64, but they are almost all misaligned.

## Important concepts

The profiler can currently only parse mach-o binary, so binary from MacOS. There's multiple binary format targeting specific arch or OS. Like ELF for linux. 
