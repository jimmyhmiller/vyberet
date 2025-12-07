import file("math-utils.arr") as Math
import file("string-utils.arr") as Str

provide: make-banner end

fun make-banner(n):
  stars = Str.repeat("*", n)
  num = Math.square(n)
  stars
end
