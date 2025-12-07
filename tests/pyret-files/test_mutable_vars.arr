# Test mutable variables and assignment

check "var binding and assignment":
  var x = 10
  x is 10
  x := 20
  x is 20
end

check "multiple var mutations":
  var counter = 0
  counter := counter + 1
  counter is 1
  counter := counter + 1
  counter is 2
  counter := counter + 5
  counter is 7
end

check "var with complex expressions":
  var total = 0
  total := (5 * 2) + 3
  total is 13
end

check "multiple vars":
  var x = 1
  var y = 2
  x := x + y
  y := x + y
  x is 3
  y is 5
end
