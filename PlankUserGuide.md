# Plank Programming Language User Guide

## Table of Contents
1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Basic Syntax](#basic-syntax)
4. [Data Types](#data-types)
5. [Variables](#variables)
6. [Operators](#operators)
7. [Control Flow](#control-flow)
8. [Functions](#functions)
9. [Input/Output](#inputoutput)
10. [Complete Example Programs](#complete-example-programs)
11. [Best Practices](#best-practices)

## Introduction

Plank is a simple, educational programming language designed for learning programming concepts. It features a clean syntax and supports fundamental programming constructs including variables, functions, control flow, and basic I/O operations.

### Key Features
- **Simple Syntax**: Easy to read and write
- **Static Typing**: `Number` and `String` data types
- **Functions**: With parameters and return values
- **Control Flow**: If statements and while loops
- **Arithmetic & Logic**: Full expression support
- **Comments**: Single-line comments for documentation

## Getting Started

### Requirements
- Rust compiler
- GCC (GNU Compiler Collection)
- Bash shell

### Installation
1. Clone the repository
2. Run `./compile.sh --setup` to set up the compiler
3. Create your first `.plank` file
4. Compile with `./compile.sh your_file.plank`

### Your First Program
```plank
/! This is a comment - your first Plank program
print "Hello, Plank!"
```

## Basic Syntax

### Comments
Use `/!` for single-line comments:
```plank
/! This is a comment
print "Hello"  /! This is also a comment
```

### Line Structure
- Each statement typically goes on its own line
- No semicolons required
- Whitespace is generally ignored

## Data Types

Plank supports two main data types:

### Number
Represents integer values:
```plank
Number age: 25
Number score: 100
Number negative: -42
```

### String
Represents text enclosed in double quotes:
```plank
String name: "Alice"
String message: "Hello, World!"
String empty: ""
```

## Variables

### Declaration
Declare variables with their type and initial value:
```plank
Number count: 0
String greeting: "Hello"
```

### Assignment
Use the `<=` operator to assign new values:
```plank
Number x: 10
x <= 20
x <= x + 5

String name: "Bob"
name <= "Alice"
```

## Operators

### Arithmetic Operators
```plank
Number a: 10
Number b: 3
Number result: 0

result <= a + b    /! Addition: 13
result <= a - b    /! Subtraction: 7
result <= a * b    /! Multiplication: 30
result <= a / b    /! Division: 3
```

### Comparison Operators
```plank
Number x: 5
Number y: 10

if x < y then             /! Less than
    print "x is smaller"
endIf

if x <= y then            /! Less than or equal
    print "x is not greater"
endIf

if x > 0 then             /! Greater than
    print "x is positive"
endIf

if x >= 0 then            /! Greater than or equal
    print "x is non-negative"
endIf

if x == 5 then            /! Equal
    print "x equals 5"
endIf

if x != y then            /! Not equal
    print "x and y are different"
endIf
```

### Logical Operators
```plank
Number age: 25
Number score: 85

/! AND operator
if age >= 18 && score >= 80 then
    print "Eligible for advanced program"
endIf

/! OR operator
if age < 18 || score < 60 then
    print "Needs additional requirements"
endIf
```

### Operator Precedence
1. Parentheses `()`
2. Multiplication `*`, Division `/`
3. Addition `+`, Subtraction `-`
4. Comparison operators `<`, `<=`, `>`, `>=`, `==`, `!=`
5. Logical AND `&&`
6. Logical OR `||`

## Control Flow

### If Statements
```plank
Number temperature: 75

if temperature > 80 then
    print "It's hot!"
endIf

/! Nested if statements
if temperature >= 60 then
    if temperature <= 80 then
        print "Perfect weather!"
    endIf
endIf
```

### While Loops
```plank
/! Simple counting loop
Number i: 1
while i <= 5 do
    print i
    i <= i + 1
endWhile

/! Nested loops
Number row: 1
while row <= 3 do
    Number col: 1
    while col <= 3 do
        print "X"
        col <= col + 1
    endWhile
    print "--- End Row ---"
    row <= row + 1
endWhile
```

## Functions

### Function Declaration
```plank
/! Function with no parameters and no return value
function sayHello():
    print "Hello from function!"
endFunction

/! Function with parameters
function greet(String name, Number age):
    print "Hello, "
    print name
    print "! You are "
    print age
    print " years old."
endFunction

/! Function with return value
function add(Number a, Number b) -> Number:
    Number sum: a + b
    return sum
endFunction
```

### Function Calls
```plank
/! Call function without parameters
sayHello()

/! Call function with parameters
greet("Alice", 25)

/! Use return value
Number result: add(10, 20)
print result
```

### Function Examples
```plank
/! Calculate factorial
function factorial(Number n) -> Number:
    Number result: 1
    Number i: 1
    while i <= n do
        result <= result * i
        i <= i + 1
    endWhile
    return result
endFunction

/! Check if number is even
function isEven(Number num) -> Number:
    Number remainder: num - (num / 2) * 2
    if remainder == 0 then
        return 1  /! True
    endIf
    return 0      /! False
endFunction
```

## Input/Output

### Print Statements
```plank
/! Print string literals
print "Hello, World!"

/! Print variables
Number age: 25
print age

String name: "Alice"
print name

/! Print multiple items (on separate lines)
print "Name: "
print name
print "Age: "
print age
```

## Complete Example Programs

### Example 1: Calculator
```plank
/! Simple calculator program
Number num1: 15
Number num2: 4
Number result: 0

print "Calculator Demo"
print "First number: "
print num1
print "Second number: "
print num2

result <= num1 + num2
print "Addition: "
print result

result <= num1 - num2
print "Subtraction: "
print result

result <= num1 * num2
print "Multiplication: "
print result

result <= num1 / num2
print "Division: "
print result
```

### Example 2: Grade Calculator
```plank
function calculateGrade(Number score) -> String:
    if score >= 90 then
        return "A"
    endIf
    if score >= 80 then
        return "B"
    endIf
    if score >= 70 then
        return "C"
    endIf
    if score >= 60 then
        return "D"
    endIf
    return "F"
endFunction

Number studentScore: 85
String grade: calculateGrade(studentScore)
print "Score: "
print studentScore
print "Grade: "
print grade
```

### Example 3: Number Pattern
```plank
function printPattern(Number rows):
    Number i: 1
    while i <= rows do
        Number j: 1
        while j <= i do
            print j
            print " "
            j <= j + 1
        endWhile
        print "--- End Row ---"
        i <= i + 1
    endWhile
endFunction

printPattern(5)
```

## Best Practices

### Code Organization
1. **Use comments** to explain complex logic
2. **Group related code** together
3. **Use meaningful variable names**
4. **Keep functions focused** on a single task

### Variable Naming
```plank
/! Good variable names
Number studentAge: 18
String firstName: "John"
Number totalScore: 95

/! Avoid unclear names
Number x: 18      /! What does x represent?
String s: "John"  /! What kind of string?
```

### Function Design
```plank
/! Good: Single responsibility
function calculateArea(Number length, Number width) -> Number:
    return length * width
endFunction

/! Good: Clear parameter names
function processStudentGrade(String studentName, Number testScore):
    print "Processing grade for: "
    print studentName
    /! ... rest of processing
endFunction
```

### Error Prevention
```plank
/! Always initialize variables
Number count: 0        /! Good
String name: ""        /! Good

/! Check bounds in loops
Number i: 1
while i <= 10 do       /! Clear termination condition
    /! ... loop body
    i <= i + 1  /! Don't forget to increment!
endWhile
```

## Common Patterns

### Counting Loop
```plank
Number counter: 1
while counter <= 10 do
    /! Do something counter times
    print counter
    counter <= counter + 1
endWhile
```

### Accumulator Pattern
```plank
Number sum: 0
Number i: 1
while i <= 100 do
    sum <= sum + i
    i <= i + 1
endWhile
print "Sum of 1 to 100: "
print sum
```

### Search Pattern
```plank
function findValue(Number target, Number searchIn) -> Number:
    if searchIn == target then
        return 1  /! Found
    endIf
    return 0      /! Not found
endFunction
```

---

This guide covers all the essential features of the Plank programming language. Start with simple programs and gradually work your way up to more complex examples. Happy coding in Plank!
