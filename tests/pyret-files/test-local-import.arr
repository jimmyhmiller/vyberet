import file("math-utils.arr") as M

provide: * end

fun test():
  x = M.square(5)
  y = M.cube(3)
  x + y
end

print(test())
