# Test block expressions

check "block with single expression":
  result = block:
    42
  end
  result is 42
end

check "block with multiple statements":
  result = block:
    x = 10
    y = 20
    x + y
  end
  result is 30
end

check "nested blocks":
  result = block:
    a = 5
    block:
      b = 10
      a + b
    end
  end
  result is 15
end

check "block with shadowing":
  x = 100
  result = block:
    shadow x = 10
    x * 2
  end
  result is 20
  x is 100
end
