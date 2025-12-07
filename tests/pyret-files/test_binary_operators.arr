# Test all binary operators

check "addition":
  1 + 2 is 3
  10 + 5 is 15
  100 + 200 is 300
end

check "subtraction":
  10 - 3 is 7
  100 - 50 is 50
  5 - 5 is 0
end

check "multiplication":
  3 * 4 is 12
  10 * 10 is 100
  7 * 8 is 56
end

check "division":
  10 / 2 is 5
  15 / 3 is 5
  100 / 4 is 25
end

check "less than":
  3 < 5 is true
  5 < 3 is false
  3 < 3 is false
end

check "greater than":
  5 > 3 is true
  3 > 5 is false
  3 > 3 is false
end

check "less than or equal":
  3 <= 5 is true
  5 <= 3 is false
  3 <= 3 is true
end

check "greater than or equal":
  5 >= 3 is true
  3 >= 5 is false
  3 >= 3 is true
end

check "equality":
  5 == 5 is true
  5 == 3 is false
  "hello" == "hello" is true
end

check "inequality":
  5 <> 3 is true
  5 <> 5 is false
  "a" <> "b" is true
end

check "boolean and":
  (true and true) is true
  (true and false) is false
  (false and true) is false
  (false and false) is false
end

check "boolean or":
  (true or true) is true
  (true or false) is true
  (false or true) is true
  (false or false) is false
end

check "complex expressions":
  (2 + 3) * 4 is 20
  (10 - 5) * 2 is 10
  ((3 + 2) * (4 - 1)) / 5 is 3
end
