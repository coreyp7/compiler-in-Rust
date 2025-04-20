#!/bin/bash

# run main.rs with provided argument of filepath ($1)

# have rust output the c code into a file into a funny dir (using $2)

# run gcc in here and compile the produced c code into an executable
cargo build

plankCompilerPath="plank_compiler"
exitEarly=0

# Obtain args from user
while [ "${1:-}" != '' ]; do
    case "$1" in
      '-s' | '--src')
        shift
        src=$1
        ;;
      '-o' | '--output')
        shift
        outputDir=$1
        ;;
    esac
    shift
  done

echo $src
echo $outputDir
echo $plankCompilerPath

# Validate all paths are valid
if [ ! -f $plankCompilerPath ]; then
    echo "plank compiler doesn't exist, path is incorrect"
    exitEarly=1
fi

if [ ! -f $src ]; then
    echo "src doesn't exist"
    exitEarly=1
fi

if [ ! -d $outputDir ]; then
    echo "output dir doesn't exist"
    exitEarly=1
fi

if [ $exitEarly -eq 1 ]; then
    echo "Exiting early."
    exit
fi

# This is where the c code will be put and compiled
cCodeDir="$outputDir/plank_target"
if [ ! -d $cCodeDir ]; then
    mkdir cCodeDir
    echo "creating cCodeDir..."
    exitEarly=1
fi

echo "Giving $src to rust file"

# Let rust do its thing
# Pass it src and outputDir
echo "Running command:"
echo "./$plankCompilerPath $src $cCodeDir"
./$plankCompilerPath $src $cCodeDir

# Ensure that the source code file was created
cCodeFile="$cCodeDir/main.c"
if [ ! -f $cCodeFile ]; then
    echo "Something went wrong when compiling your file: cannot create executable."
    echo "$cCodeFile"
    exitEarly=1
fi

echo "Created C code :D"

gcc $cCodeFile -o $outputDir/main.plank



