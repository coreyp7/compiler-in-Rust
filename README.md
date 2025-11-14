
# About
I wrote my first lanugage programming lanugage (compiler) so I could learn Rust. <br>
The language is named plank.
If you'd like to use this wonderful language, feel free. 
```
String speech: celebrate();
println(speech);

function celebrate () returns String: 
    return "hello, plank";
endFunction
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
| -i (--src) | Your plank source code |
| -o (--output) | Directory to output executable in |
| -s (--setup) | Compiles the Plank compiler in the repo you cloned (not needed after initial setup)|

## A plank example: Finding our favorite number
```
println("hello, plank");

Number favoriteNumber: 7;
Number secondFavNumber: favoriteNumber - 3;
String name: "juandissimo";

Number i: 0;
while (i != favoriteNumber):
    print(i);
    println(" is not our fav number, going to increment.");

    i <= i + 1;

    if (i == favoriteNumber):
        print ("Finally! ");
        print (i);
        println(" is my favorite number.");
    else:
        if (i == secondFavNumber):
            print(secondFavNumber);
            println(" is also a cool number, but not my favorite.");
            print("Adding them together results in the number ");
            println(add(favoriteNumber, secondFavNumber));
        endIf
    endIf
endWhile

println("What a pragmatic example of plank.");

function add (Number numOne, Number numTwo) returns Number:
    return numOne + numTwo;
endFunction
```
