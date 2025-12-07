# Pyret List Implementation (Simplified for vyberet)
# Based on trove/lists.arr

provide:
  empty, link, List,
  range, repeat,
  length, map, filter, fold, each
end

# List data type
data List:
  | empty with:
    method length(self):
      0
    end,
    method get(self, n):
      raise("Cannot get from empty list")
    end,
    method append(self, other):
      other
    end,
    method reverse(self):
      empty
    end,
    method first(self):
      raise("Cannot get first of empty list")
    end,
    method rest(self):
      raise("Cannot get rest of empty list")
    end,
    method last(self):
      raise("Cannot get last of empty list")
    end,
    method member(self, elt):
      false
    end,
    method push(self, elt):
      link(elt, self)
    end
  | link(first, rest) with:
    method length(self):
      1 + self.rest.length()
    end,
    method get(self, n):
      if n == 0:
        self.first
      else:
        self.rest.get(n - 1)
      end
    end,
    method append(self, other):
      link(self.first, self.rest.append(other))
    end,
    method reverse(self):
      fun helper(lst, acc):
        if is-empty(lst):
          acc
        else if is-link(lst):
          helper(lst.rest, link(lst.first, acc))
        end
      end
      helper(self, empty)
    end,
    method first(self):
      self.first
    end,
    method rest(self):
      self.rest
    end,
    method last(self):
      if is-empty(self.rest):
        self.first
      else:
        self.rest.last()
      end
    end,
    method member(self, elt):
      if self.first == elt:
        true
      else:
        self.rest.member(elt)
      end
    end,
    method push(self, elt):
      link(elt, self)
    end
end

# Standalone functions

fun range(start, stop):
  doc: "Create a list of numbers from start to stop-1"
  if start >= stop:
    empty
  else:
    link(start, range(start + 1, stop))
  end
where:
  range(0, 5) is link(0, link(1, link(2, link(3, link(4, empty)))))
  range(3, 3) is empty
  range(5, 3) is empty
end

fun repeat(n, e):
  doc: "Create a list with n copies of e"
  if n <= 0:
    empty
  else:
    link(e, repeat(n - 1, e))
  end
where:
  repeat(3, "x") is link("x", link("x", link("x", empty)))
  repeat(0, 5) is empty
end

fun length(lst):
  doc: "Returns the length of a list (tail-recursive)"
  fun helper(l, acc):
    if is-empty(l):
      acc
    else if is-link(l):
      helper(l.rest, acc + 1)
    end
  end
  helper(lst, 0)
where:
  length(empty) is 0
  length(link(1, empty)) is 1
  length(link(1, link(2, link(3, empty)))) is 3
end

fun map(f, lst):
  doc: "Apply function f to each element of lst"
  if is-empty(lst):
    empty
  else if is-link(lst):
    link(f(lst.first), map(f, lst.rest))
  end
where:
  map(lam(x): x + 1 end, link(1, link(2, link(3, empty)))) is
    link(2, link(3, link(4, empty)))
end

fun filter(f, lst):
  doc: "Keep only elements where f returns true"
  if is-empty(lst):
    empty
  else if is-link(lst):
    if f(lst.first):
      link(lst.first, filter(f, lst.rest))
    else:
      filter(f, lst.rest)
    end
  end
where:
  filter(lam(x): x > 2 end, link(1, link(2, link(3, link(4, empty))))) is
    link(3, link(4, empty))
end

fun fold(f, base, lst):
  doc: "Left fold over list"
  if is-empty(lst):
    base
  else if is-link(lst):
    fold(f, f(base, lst.first), lst.rest)
  end
where:
  fold(lam(acc, x): acc + x end, 0, link(1, link(2, link(3, empty)))) is 6
end

fun each(f, lst):
  doc: "Apply f to each element for side effects"
  if is-empty(lst):
    nothing
  else if is-link(lst):
    block:
      f(lst.first)
      each(f, lst.rest)
    end
  end
end

check "List tests":
  # Basic construction
  l = link(1, link(2, link(3, empty)))
  l.length() is 3
  l.first is 1
  l.rest is link(2, link(3, empty))
  l.last() is 3

  # Get
  l.get(0) is 1
  l.get(1) is 2
  l.get(2) is 3

  # Member
  l.member(2) is true
  l.member(5) is false

  # Append
  l.append(link(4, link(5, empty))) is
    link(1, link(2, link(3, link(4, link(5, empty)))))

  # Reverse
  l.reverse() is link(3, link(2, link(1, empty)))

  # Push
  l.push(0) is link(0, link(1, link(2, link(3, empty))))

  # Range
  range(0, 5).length() is 5
  range(2, 7).first is 2

  # Repeat
  repeat(3, "x").length() is 3

  # Map
  map(lam(x): x * 2 end, link(1, link(2, link(3, empty)))) is
    link(2, link(4, link(6, empty)))

  # Filter
  filter(lam(x): x > 1 end, link(1, link(2, link(3, empty)))) is
    link(2, link(3, empty))

  # Fold
  fold(lam(acc, x): acc + x end, 0, link(1, link(2, link(3, empty)))) is 6
end
