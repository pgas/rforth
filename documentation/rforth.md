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

Examples:

```forth
3 4 + .   \ pushes 3 and 4, adds to 7, prints 7
10 2 * .  \ pushes 10 and 2, multiplies to 20, prints 20
```

### 2. Stack Operations

- `dup`  : duplicate top of stack
- `drop` : remove top of stack
- `swap` : swap top two stack items
- `over` : copy second item to top

Examples:

```forth
5 dup * .       \ duplicates 5, multiplies to 25, prints 25
1 2 swap . .    \ stack [1 2] -> swap to [2 1], prints 2 then 1
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
- True is represented by -1, false by 0.

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

### 5. Comparisons

- `=` : equal
- `<` : less than
- `>` : greater than

Examples:

```forth
5 5 = .    \ pushes 5 and 5, compares equal (1), prints 1
3 7 < .    \ pushes 3 and 7, compares less (1), prints 1
```

### 6. Comments

- Line comments start with `\` and continue to the end of the line.
- Parenthesis comments are enclosed in `( ... )` and can span within a line.

Examples:
```forth
\ This is a line comment
10 20 + .    \ prints 30, comment ignored

( This is a parenthesis comment )
5 3 * .      ( multiplies 5 and 3, prints 15 )
```
