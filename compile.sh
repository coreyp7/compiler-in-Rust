#!/bin/bash

#plankCompilerPath="plank_compiler"
plankCompilerPath="./target/release/compiler"
exitEarly=0
setup=0
debug=0
#helloplank="print \"hello, plank\""
helloplank=$"
/! Declaring a variable.
Number num: 14
Number another: 1

/! Assigning to a variable.
update num <= 5

/! If conditional branch
if num > another then
    /! Here's how to print to console.
    print \"Welcome to plank. This is an example program.\"
endIf

/! Here's an example while loop
while num >= another do
    update another <= another + 1
    if num > another then
        print \"hello, \"
    endIf
endWhile

print \"world!\"

"

# Obtain args from user
while [ "${1:-}" != '' ]; do
    case "$1" in
      '-i' | '--src')
        shift
        src=$1
        ;;
      '-o' | '--output')
        shift
        outputDir=$1
        ;;
        '-s' | '--setup')
        setup=1  
    esac
    shift
  done

if [ $setup -eq 1 ]; then
    echo "SETUP: Setup flag specified; going to build compiler and setup current directory for Plank compiler."

    # TODO: need to clean up code so there's no warnings anymore
    cargo build --quiet --release
    
    # Ensure it was built correctly.
    if [ ! -f './target/release/compiler' ]; then
        echo "SETUP: Building compiler has failed; aborting."
    else
        echo "SETUP: Have successfully built Plank compiler."
        touch ./hello_world.plank
        echo "$helloplank" > hello_world.plank
        echo "SETUP: Example file 'hello_world.plank' has been created in current directory."
        echo "SETUP: This file will be compiled into an executable in the current dir, named 'plank_program.exe'."
        src=hello_world.plank
        outputDir=.
    fi
fi

# Validate all paths are valid
if [ ! -f $plankCompilerPath ]; then
    echo "Invalid arg: plank compiler doesn't exist, path is incorrect"
    exitEarly=1
fi

if [[ -z "$src" ]]; then
    echo "Missing arg; specify -i (--src) of plank source code file to compile"
    exitEarly=1
elif [ ! -f $src ]; then
    echo "Invalid arg -i (--src); file doesn't exist ($src)"
    exitEarly=1
fi

if [[ -z "$outputDir" ]]; then
    echo "Missing arg; specify -o (--output) output directory to put plank exe"
    exitEarly=1
elif [ ! -d $outputDir ]; then
    echo "Invalid arg -o (--output); directory doesn't exist ($outputDir)"
    exitEarly=1
fi


if [ $exitEarly -eq 1 ]; then
    echo "Fix incorrect args to compile."
    echo "(If you haven't ran this script with --setup, do it now)"
    exit
fi

# Let rust do its thing
# Pass it src and outputDir
$plankCompilerPath $src $outputDir

# Ensure that the source code file was created
cCodeFile="$outputDir/main.c"
if [ ! -f $cCodeFile ]; then
    echo "ERROR: Something went wrong when compiling your file: cannot create executable."
    exitEarly=1
fi

#echo "Created C code :D"

# Compile the C code into exe.
gcc $cCodeFile -o $outputDir/plank_program.exe
rm $cCodeFile

echo "Plank compiled: '$outputDir/plank_program.exe'."




