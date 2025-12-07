# Comprehensive test of all check operators

fun positive(x):
  x > 0
end

fun divide(x, y):
  if y == 0:
    raise("Division by zero")
  else:
    x / y
  end
end

check "is operator":
  1 + 1 is 2
  "hello" is "hello"
end

check "is-not operator":
  1 is-not 2
  "hello" is-not "world"
end

check "satisfies operator":
  5 satisfies positive
  10 satisfies positive
end

check "violates operator":
  -5 violates positive
  0 violates positive
end

check "raises operator":
  divide(10, 0) raises "Division by zero"
end

check "does-not-raise operator":
  divide(10, 2) does-not-raise ""
end
