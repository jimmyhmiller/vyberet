provide: repeat, join end

fun repeat(s, n):
  if n <= 0:
    ""
  else:
    s + repeat(s, n - 1)
  end
end

fun join(strings, sep):
  # TODO: implement when we have list methods
  "joined"
end
