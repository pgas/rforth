\ immediate.fth - Example of using immediate words in rforth

\ Define a custom immediate word that inserts a number during compilation
: ANSWER ( -- n )
  42  \ This number will be inserted at compile time
; IMMEDIATE

\ Using the immediate word - it will execute during compilation
: TEST-IMMEDIATE
  ANSWER +    \ ANSWER is replaced by 42 during compilation
;

\ Another immediate word to insert a 2x calculation
: DOUBLE ( n -- 2n )
  2 *  \ This calculation happens at compile time
; IMMEDIATE

\ Using DOUBLE in a definition
: QUADRUPLE 
  DOUBLE DOUBLE  \ Will be replaced by constant at compile time
;

\ To test these words
10 TEST-IMMEDIATE .   \ Should output 52 (10 + 42)
5 QUADRUPLE .         \ Should output 20 (5 * 2 * 2)

\ Define a word that is made immediate after definition
: TRIPLE ( n -- 3n )
  3 *
;
IMMEDIATE  \ Mark the latest defined word (TRIPLE) as immediate

\ Use our triple word in a definition
: TIMES-6
  TRIPLE DOUBLE  \ TRIPLE executes at compile time, DOUBLE too
;

\ Test TIMES-6
7 TIMES-6 .    \ Should output 42 (7 * 3 * 2)