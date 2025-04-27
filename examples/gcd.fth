\ gcd.fth - Euclid’s GCD example
: GCD ( a b -- gcd )
  dup 0 = if
    drop
  else
    over mod swap GCD
  then ;

48 18 GCD .  \ prints 6