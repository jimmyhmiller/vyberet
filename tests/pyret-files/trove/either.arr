# Either type

provide: left, right, Either end

data Either:
  | left(v)
  | right(v)
end

check "Either tests":
  x = left(5)
  x is left(5)

  y = right(10)
  y is right(10)
end
