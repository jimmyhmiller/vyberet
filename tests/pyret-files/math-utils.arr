provide: square, cube, sum-list end

fun square(n):
  n * n
end

fun cube(n):
  n * n * n
end

fun sum-list(nums):
  if nums.length() == 0:
    0
  else:
    nums.first() + sum-list(nums.rest())
  end
end
