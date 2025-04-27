\ examples/factorial.fth

\ Recursive factorial definition
: FACT ( n -- n! )
  dup 1 > if
    dup 1 - FACT *
  else
    drop 1
  then 
;

\ Compute and print factorial of 5
5 FACT .