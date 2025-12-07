fun helper(l, acc):
  if is-empty(l):
    acc
  else if is-link(l):
    helper(l.rest, link(l.first, acc))
  end
end

fun is-empty(x):
  false
end

fun is-link(x):
  true
end

fun link(a, b):
  0
end
