# Pick type

provide: pick-none, pick-some, Pick end

data Pick:
  | pick-none
  | pick-some(elt, rest)
end

check "Pick tests":
  x = pick-none
  x is pick-none

  y = pick-some(5, 10)
  y is pick-some(5, 10)
  y.elt is 5
  y.rest is 10
end
