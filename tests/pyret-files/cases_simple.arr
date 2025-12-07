data Maybe:
  | some(value)
  | none
end

m1 = some(42)
m2 = none()

check "simple cases":
  cases (Maybe) m1:
    | some(v) => v
    | none => 0
  end is 42

  cases (Maybe) m2:
    | some(v) => v
    | none => 0
  end is 0
end
