# Test to verify Pyret's actual operator syntax

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

check "violates (should be satisfies-not)":
  -5 violates positive
end

check "does-not-raise (should be raises-not)":
  divide(10, 2) does-not-raise ""
end
