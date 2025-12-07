data Shape:
  | circle(radius)
  | rectangle(width, height)
  | triangle(base, height)
end

fun area(s):
  cases (Shape) s:
    | circle(r) => 3 * r * r
    | rectangle(w, h) => w * h
    | triangle(b, h) => (b * h) / 2
  end
end

check "cases with fields":
  area(circle(5)) is 75
  area(rectangle(4, 6)) is 24
  area(triangle(6, 4)) is 12
end

data Color:
  | red
  | green
  | blue
end

fun color-to-number(c):
  cases (Color) c:
    | red => 0
    | green => 1
    | blue => 2
  end
end

check "singleton cases":
  color-to-number(red()) is 0
  color-to-number(green()) is 1
  color-to-number(blue()) is 2
end

data Result:
  | ok(value)
  | error(message)
end

fun unwrap-or(r, default):
  cases (Result) r:
    | ok(v) => v
    | error(_) => default
  end
end

check "cases with underscore":
  unwrap-or(ok(42), 0) is 42
  unwrap-or(error("failed"), 99) is 99
end
