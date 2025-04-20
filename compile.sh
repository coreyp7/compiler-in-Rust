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

# echo $src
# echo $outputDir
# echo $plankCompilerPath

# Validate all paths are valid
if [ ! -f $plankCompilerPath ]; then
    echo "setup error: plank compiler doesn't exist, path is incorrect"
    exitEarly=1
fi

if [[ -z "$src" ]]; then
    echo "Missing arg; specify -s (--src) of plank source code file to compile"
    exitEarly=1
elif [ ! -f $src ]; then
    echo "Invalid arg -s (--src); file doesn't exist ($src)"
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
    exit
fi

# This is where the c code will be put and compiled
#cCodeDir="$outputDir/plank_target"
#if [ ! -d $cCodeDir ]; then
    #mkdir $cCodeDir
    #echo "creating cCodeDir..."
    #exitEarly=1
#fi

#echo "Giving $src to rust file"

# Let rust do its thing
# Pass it src and outputDir
echo "Running command:"
#echo "./$plankCompilerPath $src $cCodeDir"
echo "./$plankCompilerPath $src $outputDir"
./$plankCompilerPath $src $outputDir

# Ensure that the source code file was created
#cCodeFile="$cCodeDir/main.c"
#if [ ! -f $cCodeFile ]; then
    #echo "Something went wrong when compiling your file: cannot create executable."
    #echo "$cCodeFile"
    #exitEarly=1
#fi

echo "Created C code :D"

gcc $outputDir/main.c -o $outputDir/plank_program.exe
rm $outputDir/main.c





