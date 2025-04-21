#!/bin/bash

cargo build

#plankCompilerPath="plank_compiler"
plankCompilerPath="./target/release/compiler"
exitEarly=0
setup=0
helloplank="print \"hello, plank\""

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
    echo "Setup flag specified; going to build compiler and setup current 
        directory for Plank compiler."

    cargo build --release
    
    # Ensure it was built correctly.
    if [ ! -f './target/release/compiler' ]; then
        echo "Building compiler has failed; aborting."
    else
        echo "Have successfully built Plank compiler."
        touch ./hello_world.plank
        echo $helloplank > hello_world.plank
        echo "Example file has been created in current directory, will compile
        in curr dir."
        src=hello_world.plank
        outputDir=.
    fi
fi

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

# Let rust do its thing
# Pass it src and outputDir
echo "Running command:"
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





