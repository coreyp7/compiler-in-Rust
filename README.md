# About
I wrote my first lanugage programming lanugage (compiler) so I could learn Rust. <br>
The language is named plank.
If you'd like to use this wonderful language, feel free. 
```
/! Here's an example Plank program.
/! It prints hello 25 times; then plank afterwards.
Number i: 0;
while (i < 25):
    print "hello ";
    print i;
    println "!";
    i <= i + 1;
endWhile
print "plank"
```

## Requirements
- Rust
- gcc
- run in a bash shell

## Install Instructions
1. Clone the repo
2. Run `./compile.sh --setup` <br>
*This will create an example program in your directory and compile it into an executable to ensure that setup is successful.* <br>

To compile a .plank file, run compile.sh with the following args:
| Argument | Description |
| ----------- | ----------- |
| -i (--src) | Your Plank source code |
| -o (--output) | Directory to output executable in |
| -s (--setup) | Compiles the Plank compiler in the repo you cloned (not needed after initial setup)|

## An Uncouth Programming Language "Specification"
```
/! Here's how you comment your code.

/! Declaring a variable.
Number num: 14
Number another: 1

/! Assigning to a variable.
update num <= 26

/! If conditional branch
if num > another then
    /! Here's how to print to console.
    print "wow! num is greater than that other variable"
endIf

/! Here's an example while loop
while num >= another do
    update another <= another + 1
    if num > another then
        print "still smaller"
    endIf
endWhile

print "now greater!"
```
