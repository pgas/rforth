\ examples/fibonacci.fth

\ Recursive Fibonacci definition
: fib
  dup 2 < if
    drop 1
  else
    dup 1 - fib
    swap 2 - fib
    +
  then
;

\ Compute and print fib(10)
10 fib .