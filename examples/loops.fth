\ loops.fth - Examples of DO loops and I index variable usage

\ Basic DO LOOP - prints numbers from 0 to 9
: COUNT-TO-10  
  10 0 DO
    I .
  LOOP ;

\ Calculate sum of numbers from 0 to n-1
: SUM ( n -- sum )
  0 swap  \ Initialize sum to 0, n is on top
  0 DO
    I +    \ Add current index to sum
  LOOP ;

\ Print multiplication table for n (up to 10)
: TIMES-TABLE ( n -- )
  DUP       \ Keep a copy of n
  11 1 DO
    DUP I * .  \ Print n * i
  LOOP
  DROP ;

\ Compute factorial using a DO LOOP
: FACTORIAL ( n -- n! )
  1           \ Initialize result to 1
  swap 1+ 1 DO
    I *       \ Multiply result by current index
  LOOP ;

\ Example usage
COUNT-TO-10

5 SUM .     \ Sum of 0..4 = 10
10 SUM .    \ Sum of 0..9 = 45

7 TIMES-TABLE

5 FACTORIAL .  \ 5! = 120