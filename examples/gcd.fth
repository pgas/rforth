\ gcd.fth - Euclid’s GCD example
: GCD ( a b -- gcd )
  dup 0 = if
    drop \ Base case: gcd(a, 0) = a. Drop the 0, leave a.
  else
    swap over mod GCD \ Recursive step: gcd(a, b) = gcd(b, a mod b)
  then ;

48 18 GCD .  \ prints 6
18 48 GCD .  \ prints 6
5 0 GCD .    \ prints 5
0 5 GCD .    \ prints 5