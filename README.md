# Plank Programming Lanuguage
I wrote my first lanugage programming lanugage (compiler) so I could learn Rust. <br>
If you'd like to use this wonderful language, feel free. 
```
/! Here's an example Plank program.
/! It prints hello 25 times; then plank afterwards.
Number i: 0
while i < 25 then
    print "hello, "
    update i <= i + 1
endWhile
print "plank"
```

## Requirements
- Rust
- gcc
- run in a bash shell

## Getting Started
1. Clone the repo
2. Run `./compile.sh --setup` <br>
*This will create an example program in your directory and compile it into an executable to ensure that setup is successful.* <br>

To compile a .plank file, run compile.sh with the following args:
| Argument | Description |
| ----------- | ----------- |
| -i (--src) | Your Plank source code |
| -o (--output) | Directory to output executable in |
| -s (--setup) | Compiles the Plank compiler in the repo you cloned (not needed after initial setup)|

## Programming Language Specification
tba
