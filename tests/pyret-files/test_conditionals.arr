# Test if-else expressions

check "simple if-else with true":
  result = if true: 10 else: 20 end
  result is 10
end

check "simple if-else with false":
  result = if false: 10 else: 20 end
  result is 20
end

check "if-else with comparisons":
  x = 5
  result = if x < 10: "small" else: "big" end
  result is "small"
end

check "else-if chain":
  score = 75
  grade = if score >= 90:
    "A"
  else if score >= 80:
    "B"
  else if score >= 70:
    "C"
  else:
    "F"
  end
  grade is "C"
end

check "nested if-else":
  x = 5
  y = 10
  result = if x < 10:
    if y > 5:
      "both true"
    else:
      "x only"
    end
  else:
    "neither"
  end
  result is "both true"
end

check "if-else with expressions":
  a = 3
  b = 7
  max = if a > b: a else: b end
  max is 7
end
