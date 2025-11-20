#!/bin/bash

# TODO: update this 
#plankCompilerPath="plank_compiler"

releaseDir="./target/debug"
warningsFile="${releaseDir}/warnings.txt"

plankCompilerPath="./target/debug/compiler"
#plankCompilerPath="./target/debug/compiler"

exitEarly=0
setup=0
debug=0
isSafeToCompile=0

example_file_path="./example.plank"

show_help() {
    echo "Flags:"
    echo "    -i, --src <filepath>       Plank source code file to compile"
    echo "    -o, --output <dirpath>     Output dir the executable will be created in"
    echo "    -s, --setup                Build the compiler and setup in this directory"
    echo "    -h, --help                 You already figured this one out"
    echo "Dev flags to play with:"
    echo "    -d, --debug                Enable debug mode for compilation"
    echo "    -n, --isSafeToCompile      Generate C code only, don't compile to executable"
    echo "    -D, --dev                  Use the debug version of the compiler"
    echo ""
}

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
        ;;
        '-d' | '--debug')
        debug=1
        ;;
        '-n' | '--isSafeToCompile')
        isSafeToCompile=1
        ;;
        '-D' | '--dev')
        plankCompilerPath="./target/debug/compiler"
        ;;
        '-h' | '--help')
        show_help
        exit 0
        ;;
    esac
    shift
  done

if [ $setup -eq 1 ]; then
    echo "SETUP: Setup flag specified; going to build compiler and setup current directory for Plank compiler."

    # TODO: need to clean up code so there's no warnings anymore
    touch $warningsFile
    cargo build --quiet &> $warningsFile
    
    # Ensure it was built correctly.
    if [ ! -f $plankCompilerPath ]; then
        echo "SETUP: Building compiler has failed; aborting."
    else
        echo "SETUP: Have successfully built Plank compiler."
        echo "SETUP: Compiling the 'example.plank' file into an executable as an example."
        src=example.plank
        outputDir=.
    fi
fi

# Validate all paths are valid
if [ ! -f $plankCompilerPath ]; then
    echo "Invalid arg: plank compiler doesn't exist; please run './compile.sh --setup' to build the plank compiler."
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
# Pass it src and outputDir, and optionally debug flag
if [ $debug -eq 1 ]; then
    $plankCompilerPath $src $outputDir --debug
else
    $plankCompilerPath $src $outputDir
fi

# Ensure that the c source code file was created
cCodeFile="$outputDir/main.c"
if [ ! -f $cCodeFile ]; then
    # This means there were errors when compiling.
    # The errors will be printed from Rust. 
    exit
fi

if [ $isSafeToCompile -eq 1 ]; then
    echo "not compiling; process done"
    exit
fi

# Compile the C code into exe.
if [ $debug -eq 1 ]; then
    gcc $cCodeFile -o $outputDir/plank_program.exe
else
    gcc $cCodeFile -o $outputDir/plank_program.exe -w
fi
rm $cCodeFile

echo -e "\033[32mCompilation successful: Plank program compiled to '$outputDir/plank_program.exe'\033[0m"




