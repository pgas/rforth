# rforth Documentation

## Running the Interpreter

1. Build and run with cargo:

   ```bash
   cargo run --release
   ```

2. Or run the compiled binary directly:

   ```bash
   ./target/release/rforth
   ```

Once started, you get a `>> ` prompt. Enter Forth words and press Enter to execute.

---

## Operations Implemented

### 1. Arithmetic Operations

- `+` : addition
- `-` : subtraction
- `*` : multiplication
- `/` : integer division
- `mod` : remainder (modulo)

Examples:

```forth
3 4 + .   \ pushes 3 and 4, adds to 7, prints 7
10 2 * .  \ pushes 10 and 2, multiplies to 20, prints 20
5 2 mod . \ pushes 5 and 2, computes remainder 1, prints 1
```

### 2. Stack Operations

- `dup`  : duplicate top of stack
- `drop` : remove top of stack
- `swap` : swap top two stack items
- `over` : copy second item to top
- `rot`  : rotate top three items (3rd to top)
- `?dup` : duplicate top of stack only if non-zero
- `2dup` : duplicate top two stack items
- `2drop` : remove top two stack items
- `2swap` : swap top two pairs of items
- `2over` : copy second pair to top
- `-rot`  : reverse rotate top three items

Examples:

```forth
5 dup * .       \ duplicates 5, multiplies to 25, prints 25
1 2 swap . .    \ stack [1 2] -> swap to [2 1], prints 2 then 1
1 2 3 rot . . . \ stack [1 2 3] -> rot to [2 3 1], prints 1 then 3 then 2
```

### 3. Word Definitions

- `: <name> ... ;` defines a new word.

Examples:

```forth
: SQUARE dup * ;
5 SQUARE .    \ prints 25
```

### 4. Conditionals

- Only valid inside a word definition (`: ... ;`).
- True is represented by non-zero, false by 0.

Examples:

```forth
: SHOW-IF ( n -- )
  if 42 then ;
-1 SHOW-IF .   \ true branch prints 42
0 SHOW-IF      \ false branch does nothing
```

```forth
: SHOW-IFELSE ( n -- )
  if 1 else -1 then ;
-1 SHOW-IFELSE .  \ true branch prints 1
0 SHOW-IFELSE .   \ false branch prints -1
```

### 5. Loops

- Only valid inside a word definition (`: ... ;`).
- `DO ... LOOP` : iterate from starting value (inclusive) to limit (exclusive)
- `I` : push current loop index to the stack (within a DO...LOOP)

Examples:

```forth
: COUNT-TO-10
  10 0 DO
    I .  \ print current index
  LOOP ;
COUNT-TO-10  \ prints 0 1 2 3 4 5 6 7 8 9

\ Sum numbers from 0 to n-1
: SUM ( n -- sum )
  0 swap  \ start with sum=0
  0 DO
    I +    \ add current index to sum
  LOOP ;
5 SUM .  \ 0+1+2+3+4 = 10, prints 10
```

### 6. Comparisons

- `=` : equal
- `<` : less than
- `>` : greater than

Examples:

```forth
5 5 = .    \ pushes 5 and 5, compares equal (-1), prints -1
3 7 < .    \ pushes 3 and 7, compares less (-1), prints -1
```

### 7. Comments

- Line comments start with `\` and continue to the end of the line.
- Parenthesis comments are enclosed in `( ... )` and can span within a line.

Examples:
```forth
\ This is a line comment
10 20 + .    \ prints 30, comment ignored

( This is a parenthesis comment )
5 3 * .      ( multiplies 5 and 3, prints 15 )
```

### 8. Immediate Words

Immediate words are executed at compile time rather than at runtime. They are useful for meta-programming and implementing custom control structures.

- `IMMEDIATE` : marks the most recently defined word as immediate
- A word can also be defined as immediate by adding `IMMEDIATE` directly after the definition

Examples:

```forth
\ Define a word that is immediate from the start
: ANSWER ( -- n )
  42  \ This number will be inserted at compile time
; IMMEDIATE

\ Using the immediate word - it will execute during compilation
: TEST-IMMEDIATE
  ANSWER +    \ ANSWER is replaced by 42 during compilation
;

10 TEST-IMMEDIATE .   \ Prints 52 (10 + 42)

\ Make a word immediate after its definition
: TRIPLE ( n -- 3n )
  3 *
;
IMMEDIATE  \ Mark TRIPLE as immediate

\ Use TRIPLE in a definition - it executes at compile time
: TIMES-6
  TRIPLE DOUBLE  \ TRIPLE executes at compile time
;

7 TIMES-6 .    \ Prints 42 (7 * 3 * 2)
```

How Immediate Words Work:
- Normal words are compiled into a definition and executed when the definition is called
- Immediate words are executed right away during compilation
- This allows for compile-time calculations and code transformations
- The `IMMEDIATE` flag is stored with each word in the dictionary

Use Cases:
- Custom control structures
- Compile-time optimizations
- Code generation
- Domain-specific language features
