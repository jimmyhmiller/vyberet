# Test tuple expressions

check "simple tuple creation":
  t = {1; 2; 3}
  t.{0} is 1
  t.{1} is 2
  t.{2} is 3
end

check "tuple with different types":
  t = {42; "hello"; true}
  t.{0} is 42
  t.{1} is "hello"
  t.{2} is true
end

check "nested tuples":
  t = {{1; 2}; {3; 4}}
  t.{0}.{0} is 1
  t.{0}.{1} is 2
  t.{1}.{0} is 3
  t.{1}.{1} is 4
end

check "tuple in expressions":
  pair = {10; 20}
  sum = pair.{0} + pair.{1}
  sum is 30
end

check "tuple from function":
  fun make-pair(a, b):
    {a; b}
  end
  p = make-pair(5, 10)
  p.{0} is 5
  p.{1} is 10
end
