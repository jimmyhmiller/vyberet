;;; Pyret Runtime Library for R4RS Scheme
;;; Provides essential functions and data structures for compiled Pyret code

;; ============================================================================
;; Preserve Scheme Built-ins
;; ============================================================================

;; Save Scheme's built-in functions before user code can shadow them
;; This is needed because runtime functions rely on them
(define scheme:length length)
(define scheme:map map)

;; ============================================================================
;; Polymorphic Operators
;; ============================================================================

;; Pyret's + operator works for both numbers and strings
(define (pyret:+ a b)
  (cond
    ((and (number? a) (number? b)) (+ a b))
    ((and (string? a) (string? b)) (string-append a b))
    ((and (string? a) (number? b)) (string-append a (number->string b)))
    ((and (number? a) (string? b)) (string-append (number->string a) b))
    (else (error "Invalid types for +"))))

;; ============================================================================
;; Type Predicates
;; ============================================================================

(define (pyret:Number? x)
  (number? x))

(define (pyret:String? x)
  (string? x))

(define (pyret:Boolean? x)
  (boolean? x))

(define (pyret:Function? x)
  (procedure? x))

;; ============================================================================
;; Rational Number Support
;; ============================================================================

;; Rational number predicate
;; For Chicken/Gambit/Chez: checks native rationals
;; For Ribbit: will be overridden in ribbit-additions.scm to check tagged pairs
(define (pyret:rat? x)
  (and (rational? x) (not (integer? x))))

;; Rational equality (for native rationals)
;; For Ribbit: will be overridden in ribbit-additions.scm
(define (rat= x y)
  (equal? x y))

;; Constructor stub for native rationals
;; For Ribbit: will be overridden in ribbit-additions.scm
(define (make-rat n d)
  (/ n d))

;; ============================================================================
;; Equality Operations
;; ============================================================================

;; Pyret uses structural equality by default
;; For now, we use equal? which does structural comparison
;; Also handles rational numbers (both native and Ribbit custom implementation)
(define (pyret:equal? x y)
  (cond
    ; Both are rationals - use rational equality
    ((and (pyret:rat? x) (pyret:rat? y)) (rat= x y))
    ; One is rational, one is integer - compare numerically
    ((and (pyret:rat? x) (number? y)) (rat= x (make-rat y 1)))
    ((and (number? x) (pyret:rat? y)) (rat= (make-rat x 1) y))
    ; Otherwise use standard equality
    (else (equal? x y))))

(define (pyret:not-equal? x y)
  (not (equal? x y)))

;; Roughly equal (for floating point comparison)
;; Default tolerance: 1e-5
(define pyret:roughly-equal-tolerance 0.00001)

(define (pyret:roughly-equal? x y)
  (cond
    ((and (number? x) (number? y))
     (< (abs (- x y)) pyret:roughly-equal-tolerance))
    (else (equal? x y))))

;; Spaceship operator (three-way comparison)
;; Returns: ~-1 if x < y, ~0 if x == y, ~1 if x > y
(define (pyret:spaceship x y)
  (cond
    ((< x y) -1)
    ((> x y) 1)
    (else 0)))

;; ============================================================================
;; Box Operations (for mutable variables)
;; ============================================================================

;; Boxes are used to implement Pyret's mutable variables (var)
;; A box is a single-element vector

(define (make-box value)
  (vector value))

(define (box-ref box)
  (vector-ref box 0))

(define (box-set! box value)
  (vector-set! box 0 value))

(define (box? x)
  (and (vector? x) (= (vector-length x) 1)))

;; ============================================================================
;; List Operations
;; ============================================================================

;; Pyret lists compile to Scheme lists
;; Provide helper functions for list construction

(define (pyret:list . args)
  args)

;; Empty list constant (tagged)
(define pyret:empty '(pyret-list))

(define (pyret:empty? lst)
  (or (null? lst)
      (and (pair? lst)
           (eq? (car lst) 'pyret-list)
           (null? (cdr lst)))))

(define (pyret:link first rest)
  ;; If rest is a tagged pyret-list, add element to the tagged list
  ;; Otherwise, create a simple cons (for backwards compatibility)
  (if (and (pair? rest) (eq? (car rest) 'pyret-list))
      (cons 'pyret-list (cons first (cdr rest)))
      (cons first rest)))

(define (pyret:first lst)
  (if (null? lst)
      (error "Cannot get first of empty list")
      (car lst)))

(define (pyret:rest lst)
  (if (null? lst)
      (error "Cannot get rest of empty list")
      (cdr lst)))

;; ============================================================================
;; List Methods (built-in list operations)
;; ============================================================================

;; Lists in Pyret are tagged with 'pyret-list
;; These methods work on tagged lists and return appropriate values

(define (pyret:list-length lst)
  "Returns the number of elements in the list"
  (if (pyret:is-list? lst)
      (scheme:length (cdr lst))  ; Skip the 'pyret-list tag
      (pyret:error "length() requires a list")))

(define (pyret:list-first lst)
  "Returns the first element of the list, or error if empty"
  (if (not (pyret:is-list? lst))
      (pyret:error "first() requires a list")
      (let ((items (cdr lst)))
        (if (null? items)
            (pyret:error "Cannot get first of empty list")
            (car items)))))

(define (pyret:list-rest lst)
  "Returns a new list with all elements except the first"
  (if (not (pyret:is-list? lst))
      (pyret:error "rest() requires a list")
      (let ((items (cdr lst)))
        (if (null? items)
            (pyret:error "Cannot get rest of empty list")
            (cons 'pyret-list (cdr items))))))

(define (pyret:list-get lst index)
  "Returns the element at the given index (0-based)"
  (if (not (pyret:is-list? lst))
      (pyret:error "get() requires a list")
      (let ((items (cdr lst))  ; Skip the tag
            (len (scheme:length (cdr lst))))
        (if (or (< index 0) (>= index len))
            (pyret:error (string-append "List index "
                                       (number->string index)
                                       " out of bounds (length: "
                                       (number->string len)
                                       ")"))
            (list-ref items index)))))

(define (pyret:list-reverse lst)
  "Returns a new list with elements in reverse order"
  (if (not (pyret:is-list? lst))
      (pyret:error "reverse() requires a list")
      (cons 'pyret-list (reverse (cdr lst)))))

(define (pyret:list-append lst other)
  "Returns a new list with elements of other appended to lst"
  (if (not (pyret:is-list? lst))
      (pyret:error "append() requires a list")
      (if (not (pyret:is-list? other))
          (pyret:error "append() requires a list as argument")
          (cons 'pyret-list (append (cdr lst) (cdr other))))))

;; ============================================================================
;; Method Call Dispatch
;; ============================================================================

;; Universal method call dispatcher
;; Handles method calls on built-in types: lists, strings, numbers, objects, etc.
;; Usage: (pyret:method-call obj "method-name" arg1 arg2 ...)
(define (pyret:method-call obj method-name . args)
  (cond
    ;; Lists
    ((pyret:is-list? obj)
     (cond
       ((string=? method-name "length")
        (if (not (null? args))
            (pyret:error "length() takes no arguments")
            (pyret:list-length obj)))
       ((string=? method-name "first")
        (if (not (null? args))
            (pyret:error "first() takes no arguments")
            (pyret:list-first obj)))
       ((string=? method-name "rest")
        (if (not (null? args))
            (pyret:error "rest() takes no arguments")
            (pyret:list-rest obj)))
       ((string=? method-name "get")
        (if (not (= (length args) 1))
            (pyret:error "get() requires exactly 1 argument")
            (pyret:list-get obj (car args))))
       ((string=? method-name "reverse")
        (if (not (null? args))
            (pyret:error "reverse() takes no arguments")
            (pyret:list-reverse obj)))
       ((string=? method-name "append")
        (if (not (= (length args) 1))
            (pyret:error "append() requires exactly 1 argument")
            (pyret:list-append obj (car args))))
       (else
        (pyret:error (string-append "Unknown list method: " method-name)))))

    ;; Strings
    ((string? obj)
     (cond
       ((string=? method-name "length")
        (if (not (null? args))
            (pyret:error "length() takes no arguments")
            (pyret:string-length obj)))
       ((string=? method-name "substring")
        (if (not (= (length args) 2))
            (pyret:error "substring() requires exactly 2 arguments")
            (string-substring obj (car args) (cadr args))))
       ((string=? method-name "char-at")
        (if (not (= (length args) 1))
            (pyret:error "char-at() requires exactly 1 argument")
            (string-char-at obj (car args))))
       ((string=? method-name "split")
        (if (not (= (length args) 1))
            (pyret:error "split() requires exactly 1 argument")
            (string-split obj (car args))))
       ((string=? method-name "contains")
        (if (not (= (length args) 1))
            (pyret:error "contains() requires exactly 1 argument")
            (string-contains obj (car args))))
       ((string=? method-name "to-upper")
        (if (not (null? args))
            (pyret:error "to-upper() takes no arguments")
            (string-to-upper obj)))
       ((string=? method-name "to-lower")
        (if (not (null? args))
            (pyret:error "to-lower() takes no arguments")
            (string-to-lower obj)))
       ((string=? method-name "repeat")
        (if (not (= (length args) 1))
            (pyret:error "repeat() requires exactly 1 argument")
            (string-repeat obj (car args))))
       ((string=? method-name "trim")
        (if (not (null? args))
            (pyret:error "trim() takes no arguments")
            (string-trim obj)))
       (else
        (pyret:error (string-append "Unknown string method: " method-name)))))

    ;; Objects (user-defined objects with methods)
    ((pyret:is-object? obj)
     (let ((method (pyret:object-get obj method-name)))
       (if (procedure? method)
           (apply method args)
           (pyret:error (string-append "Not a method: " method-name)))))

    ;; Unknown type
    (else
     (pyret:error (string-append "Cannot call method on this type: " method-name)))))

;; ============================================================================
;; Error Handling
;; ============================================================================

(define (pyret:error msg)
  (display "Pyret error: ")
  (display msg)
  (newline)
  (error msg))

(define (pyret:raise msg)
  (pyret:error msg))

;; ============================================================================
;; Printing/Display
;; ============================================================================

;; Helper to check if something is a Pyret list (tagged with 'pyret-list)
(define (pyret:is-list? x)
  (and (pair? x) (eq? (car x) 'pyret-list)))

;; Helper to check if something is a Pyret set (tagged with 'pyret-set)
(define (pyret:is-set? x)
  (and (pair? x) (eq? (car x) 'pyret-set)))

;; Helper to check if something is a Pyret object (tagged with 'pyret-object)
(define (pyret:is-object? x)
  (and (pair? x) (eq? (car x) 'pyret-object)))

;; Helper to check if something is a data variant (tagged list with symbol tag)
;; Data variants have a symbol as the first element, but not 'pyret-list, 'pyret-set, or 'pyret-object
(define (pyret:is-data-variant? x)
  (and (pair? x)
       (symbol? (car x))
       (not (eq? (car x) 'pyret-list))
       (not (eq? (car x) 'pyret-set))
       (not (eq? (car x) 'pyret-object))
       (not (eq? (car x) 'RATIONAL))))  ; Also not a Ribbit rational

;; Helper to print a data variant
(define (pyret:print-data-variant v)
  (let ((tag (car v))
        (fields (cdr v)))
    (display tag)
    (if (not (null? fields))
        (begin
          (display "(")
          (let loop ((items fields) (first? #t))
            (if (null? items)
                (display ")")
                (begin
                  (if (not first?) (display ", "))
                  (pyret:print-value (car items))
                  (loop (cdr items) #f))))))))

;; Helper to print a Pyret list with [list: ...] syntax
(define (pyret:print-list lst)
  (display "[list: ")
  (let loop ((items (cdr lst)) (first? #t))  ; Skip the tag
    (if (null? items)
        (display "]")
        (begin
          (if (not first?) (display ", "))
          (pyret:print-value (car items))
          (loop (cdr items) #f)))))

;; Helper to print a Pyret set with [list-set: ...] syntax
;; (Pyret uses list-set for the basic set implementation)
(define (pyret:print-set s)
  (display "[list-set: ")
  (let loop ((items (cdr s)) (first? #t))  ; Skip the tag
    (if (null? items)
        (display "]")
        (begin
          (if (not first?) (display ", "))
          (pyret:print-value (car items))
          (loop (cdr items) #f)))))

;; Helper to print a Pyret object with {field: value, ...} syntax
(define (pyret:print-object obj)
  (display "{")
  (let loop ((items (cdr obj)) (first? #t))  ; Skip the tag
    (if (null? items)
        (display "}")
        (let ((field (car items)))
          (if (not first?) (display ", "))
          (display (car field))
          (display ": ")
          (pyret:print-value (cdr field))
          (loop (cdr items) #f)))))

;; Display a rational number (stub for native rationals)
;; For Ribbit: will be overridden in ribbit-additions.scm
(define (display-rat x)
  (display x))  ; Native rationals display themselves correctly

;; Helper to print individual values (recursive)
(define (pyret:print-value x)
  (cond
    ((eq? x #t) (display "true"))
    ((eq? x #f) (display "false"))
    ((pyret:is-list? x) (pyret:print-list x))
    ((pyret:is-set? x) (pyret:print-set x))
    ((pyret:is-object? x) (pyret:print-object x))
    ((pyret:is-data-variant? x) (pyret:print-data-variant x))
    ((pyret:rat? x) (display-rat x))
    (else (display x))))

;; Pyret's print() does NOT add newlines - it just displays the value
;; The value is converted to a string representation:
;; - Booleans: "true" or "false"
;; - Numbers: their string representation
;; - Strings: the string itself (no quotes)
;; - Lists: [list: elem1, elem2, ...]
(define (pyret:print x)
  (pyret:print-value x)
  x)

(define (pyret:display x)
  (display x)
  x)

;; ============================================================================
;; Data Variant Field Access (Phase 6+)
;; ============================================================================

;; Helper to get a field from a data variant by name and index
;; This is a runtime helper for data declarations
;; Variants are represented as: (list 'tag field1 field2 ...)
;; Field index is 0-based (0 = first field after tag)
(define (pyret:data-field obj field-index)
  (if (and (pair? obj) (> (scheme:length obj) (+ field-index 1)))
      (list-ref obj (+ field-index 1))  ; +1 to skip the tag
      (pyret:error (string-append "Field access out of bounds: index " (number->string field-index)))))

;; ============================================================================
;; Object/Record Support (Phase 6)
;; ============================================================================

;; Objects are represented as vectors with a tag and field values
;; Format: #(tag field1 field2 ... method1 method2 ...)

(define (pyret:make-object tag . fields)
  (let ((obj (make-vector (+ 1 (scheme:length fields)))))
    (vector-set! obj 0 tag)
    (let loop ((i 1) (fs fields))
      (if (null? fs)
          obj
          (begin
            (vector-set! obj i (car fs))
            (loop (+ i 1) (cdr fs)))))))

(define (pyret:object-tag obj)
  (vector-ref obj 0))

(define (pyret:object-field obj index)
  (vector-ref obj (+ index 1)))

(define (pyret:object-set-field! obj index value)
  (vector-set! obj (+ index 1) value))

;; ============================================================================
;; Data Constructor Support (Phase 5)
;; ============================================================================

;; Data constructors create tagged vectors
;; Variants compile to constructor functions that create these

(define (pyret:variant tag . fields)
  (apply pyret:make-object (cons tag fields)))

(define (pyret:variant-tag variant)
  (pyret:object-tag variant))

(define (pyret:variant-field variant index)
  (pyret:object-field variant index))

(define (pyret:is-variant? obj expected-tag)
  (and (vector? obj)
       (> (vector-length obj) 0)
       (equal? (vector-ref obj 0) expected-tag)))

;; ============================================================================
;; Tuple Support (Phase 4)
;; ============================================================================

;; Tuples are just Scheme vectors
;; These are convenience wrappers

(define (pyret:tuple . elements)
  (list->vector elements))

(define (pyret:tuple-get tup index)
  (vector-ref tup index))

;; ============================================================================
;; Construct Expressions (Phase 7)
;; ============================================================================

;; [list: ...] constructor
;; Tag lists with a special marker so we can distinguish from sets
(define (pyret:construct-list . elements)
  (cons 'pyret-list elements))

;; [list-set: ...] constructor - O(n²) operations
;; Tag sets with a special marker
;; Removes duplicates while preserving order
(define (pyret:construct-list-set . elements)
  ;; Remove duplicates, keeping first occurrence
  (let loop ((result '()) (items elements))
    (if (null? items)
        (cons 'pyret-set result)
        (if (member (car items) result)
            (loop result (cdr items))
            (loop (append result (list (car items))) (cdr items))))))

;; ============================================================================
;; String Operations
;; ============================================================================

(define (pyret:string-append . strings)
  (apply string-append strings))

(define (pyret:string-length str)
  (string-length str))

(define (pyret:string-get str index)
  (string-ref str index))

;; Priority 1 String Primitives (for trove support)
;; Note: string-length and string-append are R4RS builtins, we just provide wrappers

(define (string-substring str start end)
  "Returns substring from start (inclusive) to end (exclusive)"
  (if (not (string? str))
      (error "string-substring requires a string")
      (let ((len (string-length str)))
        (if (or (< start 0) (> end len) (> start end))
            (error "string-substring: invalid indices")
            (substring str start end)))))

(define (string-char-at str n)
  "Returns the character at index n as a one-character string"
  (if (not (string? str))
      (error "string-char-at requires a string")
      (let ((len (string-length str)))
        (if (or (< n 0) (>= n len))
            (error "string-char-at: index out of bounds")
            (string (string-ref str n))))))

(define (string-equal s1 s2)
  "Checks if two strings are equal"
  (if (and (string? s1) (string? s2))
      (string=? s1 s2)
      #f))

(define (string-to-upper str)
  "Converts string to uppercase"
  (if (not (string? str))
      (error "string-to-upper requires a string")
      (list->string
       (scheme:map (lambda (c)
              (if (and (char>=? c #\a) (char<=? c #\z))
                  (integer->char (- (char->integer c) 32))
                  c))
            (string->list str)))))

(define (string-to-lower str)
  "Converts string to lowercase"
  (if (not (string? str))
      (error "string-to-lower requires a string")
      (list->string
       (scheme:map (lambda (c)
              (if (and (char>=? c #\A) (char<=? c #\Z))
                  (integer->char (+ (char->integer c) 32))
                  c))
            (string->list str)))))

(define (string-repeat str n)
  "Repeats a string n times"
  (if (not (string? str))
      (error "string-repeat requires a string")
      (if (not (and (integer? n) (>= n 0)))
          (error "string-repeat requires a non-negative integer")
          (let loop ((i 0) (result ""))
            (if (>= i n)
                result
                (loop (+ i 1) (string-append result str)))))))

(define (string-trim str)
  "Removes leading and trailing whitespace from a string"
  (if (not (string? str))
      (error "string-trim requires a string")
      (let ((len (string-length str)))
        (if (= len 0)
            str
            (let* ((chars (string->list str))
                   ;; Find first non-whitespace character
                   (start (let loop ((i 0) (lst chars))
                           (cond
                             ((null? lst) i)
                             ((char-whitespace? (car lst)) (loop (+ i 1) (cdr lst)))
                             (else i))))
                   ;; Find last non-whitespace character
                   (end (let loop ((i (- len 1)) (lst (reverse chars)))
                         (cond
                           ((< i 0) -1)
                           ((char-whitespace? (car lst)) (loop (- i 1) (cdr lst)))
                           (else i)))))
              (if (or (>= start len) (< end 0) (> start end))
                  ""
                  (substring str start (+ end 1))))))))

(define (string-contains str substr)
  "Checks if str contains substr"
  (if (not (and (string? str) (string? substr)))
      (error "string-contains requires two strings")
      (let ((str-len (string-length str))
            (sub-len (string-length substr)))
        (let loop ((i 0))
          (cond
            ((> (+ i sub-len) str-len) #f)
            ((string=? (substring str i (+ i sub-len)) substr) #t)
            (else (loop (+ i 1))))))))

(define (string-split str delim)
  "Splits string by delimiter, returns Pyret list of strings"
  (if (not (and (string? str) (string? delim)))
      (error "string-split requires two strings")
      (let ((str-len (string-length str))
            (delim-len (string-length delim)))
        (if (= delim-len 0)
            (pyret:construct-list str)  ; Empty delimiter returns original string as single element
            (let loop ((start 0) (i 0) (result '()))
              (cond
                ((= i str-len)
                 (pyret:wrap-list (reverse (cons (substring str start i) result))))
                ((and (<= (+ i delim-len) str-len)
                      (string=? (substring str i (+ i delim-len)) delim))
                 (loop (+ i delim-len) (+ i delim-len)
                       (cons (substring str start i) result)))
                (else (loop start (+ i 1) result))))))))

(define (string-to-number str)
  "Converts string to number, returns some(n) or none"
  (if (not (string? str))
      (error "string-to-number requires a string")
      (let ((num (string->number str)))
        (if num
            (some num)
            (none)))))

;; ============================================================================
;; Numeric Operations
;; ============================================================================

;; R4RS may not have full numeric tower, provide fallbacks

(define (pyret:numerator n)
  (if (rational? n)
      (numerator n)
      (error "numerator requires a rational number")))

(define (pyret:denominator n)
  (if (rational? n)
      (denominator n)
      (error "denominator requires a rational number")))

;; Integer division operations
;; These are essential for algorithms that need integer arithmetic

(define (pyret:num-quotient a b)
  "Integer division - returns the quotient of a/b (floor division)"
  (quotient a b))

(define (pyret:num-remainder a b)
  "Integer division - returns the remainder of a/b"
  (remainder a b))

(define (pyret:num-modulo a b)
  "Modulo operation - returns a mod b"
  (modulo a b))

;; Floor, ceiling, truncate for converting rationals to integers
(define (pyret:num-floor n)
  "Returns the largest integer not greater than n"
  (if (number? n)
      (floor n)
      (error "floor requires a number")))

(define (pyret:num-ceiling n)
  "Returns the smallest integer not less than n"
  (if (number? n)
      (ceiling n)
      (error "ceiling requires a number")))

(define (pyret:num-truncate n)
  "Returns the integer closest to n whose absolute value is not larger"
  (if (number? n)
      (truncate n)
      (error "truncate requires a number")))

(define (pyret:num-round n)
  "Returns the closest integer to n, rounding to even when halfway"
  (if (number? n)
      (round n)
      (error "round requires a number")))

;; Priority 1 Number Primitives (for trove support)

(define (num-abs n)
  "Returns the absolute value of n"
  (abs n))

(define (num-sqrt n)
  "Returns the square root of n"
  (sqrt n))

(define (num-sqr n)
  "Returns n squared"
  (* n n))

(define (num-max n m)
  "Returns the maximum of n and m"
  (max n m))

(define (num-min n m)
  "Returns the minimum of n and m"
  (min n m))

(define (num-to-string n)
  "Converts number to string"
  (number->string n))

(define (num-random n)
  "Returns random integer from 0 to n-1"
  (if (<= n 0)
      (error "num-random requires positive integer")
      (floor (* (random) n))))

(define (num-modulo n m)
  "Modulo operation"
  (pyret:num-modulo n m))

(define (num-remainder n m)
  "Remainder operation"
  (pyret:num-remainder n m))

(define (num-floor n)
  "Floor function"
  (pyret:num-floor n))

(define (num-ceiling n)
  "Ceiling function"
  (pyret:num-ceiling n))

(define (num-truncate n)
  "Truncate function"
  (pyret:num-truncate n))

(define (num-round n)
  "Round function"
  (pyret:num-round n))

;; ============================================================================
;; RawArray Primitives (for trove support)
;; ============================================================================

;; RawArrays are mutable arrays, implemented as Scheme vectors
;; These are used internally by Pyret for efficient array operations

(define (raw-array-of val len)
  "Create a raw array of length len filled with val"
  (make-vector len val))

(define (raw-array-build f len)
  "Create a raw array by calling f(index) for each index from 0 to len-1"
  (let ((arr (make-vector len)))
    (let loop ((i 0))
      (if (< i len)
          (begin
            (vector-set! arr i (f i))
            (loop (+ i 1)))
          arr))))

(define (raw-array-get arr ix)
  "Get element at index ix from raw array"
  (vector-ref arr ix))

(define (raw-array-set arr ix val)
  "Set element at index ix in raw array (mutates!)"
  (vector-set! arr ix val))

(define (raw-array-length arr)
  "Get length of raw array"
  (vector-length arr))

(define (raw-array-to-list arr)
  "Convert raw array to Pyret list"
  (pyret:wrap-list (vector->list arr)))

(define (raw-array-from-list lst)
  "Convert Pyret list to raw array"
  (list->vector (pyret:unwrap-list lst)))

(define (raw-array-map f arr)
  "Map function f over raw array, returns new raw array"
  (let* ((len (vector-length arr))
         (result (make-vector len)))
    (let loop ((i 0))
      (if (< i len)
          (begin
            (vector-set! result i (f (vector-ref arr i)))
            (loop (+ i 1)))
          result))))

(define (raw-array-filter f arr)
  "Filter raw array by predicate f, returns new raw array"
  (let* ((len (vector-length arr))
         (temp (make-vector len))
         (count 0))
    ; First pass: collect matching elements
    (let loop ((i 0))
      (if (< i len)
          (let ((elem (vector-ref arr i)))
            (if (f elem)
                (begin
                  (vector-set! temp count elem)
                  (set! count (+ count 1))))
            (loop (+ i 1)))))
    ; Second pass: create result array of correct size
    (let ((result (make-vector count)))
      (let loop ((i 0))
        (if (< i count)
            (begin
              (vector-set! result i (vector-ref temp i))
              (loop (+ i 1)))
            result)))))

(define (raw-array-fold f init arr start)
  "Fold over raw array from index start to end"
  (let ((len (vector-length arr)))
    (let loop ((i start) (acc init))
      (if (< i len)
          (loop (+ i 1) (f acc (vector-ref arr i)))
          acc))))

;; ============================================================================
;; Check Block Support
;; ============================================================================

;; Global state for tracking check results
(define *check-tests-passed* 0)
(define *check-tests-failed* 0)
(define *check-test-results* '())  ; List of (line col result reason) tuples
(define *check-current-block-name* #f)
(define *check-current-block-file* #f)
(define *check-current-block-start-line* #f)
(define *check-current-block-start-col* #f)
(define *check-current-block-end-line* #f)
(define *check-current-block-end-col* #f)

;; Initialize a check block
(define (pyret:check-block-start name file start-line start-col end-line end-col)
  (set! *check-current-block-name* name)
  (set! *check-current-block-file* file)
  (set! *check-current-block-start-line* start-line)
  (set! *check-current-block-start-col* start-col)
  (set! *check-current-block-end-line* end-line)
  (set! *check-current-block-end-col* end-col))

;; Record a test result with location and reason
(define (pyret:check-test-result passed? line col left-val right-val reason)
  (if passed?
      (set! *check-tests-passed* (+ *check-tests-passed* 1))
      (set! *check-tests-failed* (+ *check-tests-failed* 1)))
  ;; Store result for later reporting
  (set! *check-test-results*
        (append *check-test-results*
                (list (list passed? line col left-val right-val reason)))))

;; Finish all check blocks and print summary
(define (pyret:check-block-end)
  (let ((total (+ *check-tests-passed* *check-tests-failed*)))
    (cond
      ;; No tests at all
      ((= total 0)
       (display "The program didn't define any tests.")
       (newline))
      ;; All tests passed
      ((= *check-tests-failed* 0)
       (display "Looks shipshape, all ")
       (display total)
       (display " tests passed, mate!")
       (newline))
      ;; Some tests failed
      (else
       (newline)
       (newline)
       (display "file://")
       (display *check-current-block-file*)
       (display ":")
       (display *check-current-block-start-line*)
       (display ":")
       (display *check-current-block-start-col*)
       (display "-")
       (display *check-current-block-end-line*)
       (display ":")
       (display *check-current-block-end-col*)
       (display ": ")
       (if *check-current-block-name*
           (display *check-current-block-name*)
           (display "check"))
       (display " (")
       (display *check-tests-passed*)
       (display "/")
       (display total)
       (display ") ")
       (newline)
       (newline)
       ;; Print individual test results
       (let loop ((results *check-test-results*))
         (if (not (null? results))
             (let* ((result (car results))
                    (passed? (car result))
                    (line (cadr result))
                    (col (caddr result))
                    (left-val (cadddr result))
                    (right-val (car (cddddr result)))
                    (reason (cadr (cddddr result))))
               (display "  line ")
               (display line)
               (display ", column ")
               (display col)
               (display ": ")
               (if passed?
                   (display "ok")
                   (begin
                     (display "failed because: ")
                     (newline)
                     (display "    ")
                     (newline)
                     (display reason)
                     (display " ")
                     (display left-val)
                     (display " ")
                     (display right-val)))
               (newline)
               (loop (cdr results)))))
       (newline)
       (display "Passed: ")
       (display *check-tests-passed*)
       (display "; Failed: ")
       (display *check-tests-failed*)
       (display "; Ended in Error: 0; Total: ")
       (display total)
       (newline)
       (newline)))))

;; ============================================================================
;; Helper functions for check operators
;; ============================================================================

;; Rough equality for numbers (within tolerance)
(define (pyret:is-roughly left right)
  (cond
    ((and (number? left) (number? right))
     (let ((tolerance 1e-10))
       (< (abs (- left right)) tolerance)))
    (else
     (pyret:equal? left right))))

;; Check if value satisfies predicate
(define (pyret:satisfies value predicate)
  (predicate value))

;; Exception handling using call/cc (pure R4RS Scheme)
;; This avoids Chicken-specific exception objects

(define *error-handler* #f)

(define (pyret:raise msg)
  (if *error-handler*
      (*error-handler* msg)
      (error "Uncaught exception: " msg)))

(define (pyret:catch-exception thunk)
  (call-with-current-continuation
    (lambda (k)
      (let ((old-handler *error-handler*))
        (set! *error-handler* (lambda (msg) (k (list 'error msg))))
        (let ((result (thunk)))
          (set! *error-handler* old-handler)
          (list 'ok result))))))

;; ============================================================================
;; For Loop Support (Iteration Functions)
;; ============================================================================

;; Helper to unwrap Pyret list (remove 'pyret-list tag)
(define (pyret:unwrap-list lst)
  (if (and (pair? lst) (eq? (car lst) 'pyret-list))
      (cdr lst)
      lst))

;; Helper to wrap Scheme list as Pyret list (add 'pyret-list tag)
(define (pyret:wrap-list lst)
  (cons 'pyret-list lst))

;; map - apply function to each element and collect results
;; Works with Pyret tagged lists
(define (map f lst)
  (let ((unwrapped (pyret:unwrap-list lst)))
    (let loop ((items unwrapped))
      (if (null? items)
          (pyret:wrap-list '())
          (let ((rest-result (loop (cdr items))))
            (pyret:wrap-list (cons (f (car items)) (pyret:unwrap-list rest-result))))))))

;; filter - keep only elements that satisfy predicate
;; Works with Pyret tagged lists
(define (filter pred lst)
  (let ((unwrapped (pyret:unwrap-list lst)))
    (let loop ((items unwrapped))
      (cond
        ((null? items) (pyret:wrap-list '()))
        ((pred (car items))
         (let ((rest-result (loop (cdr items))))
           (pyret:wrap-list (cons (car items) (pyret:unwrap-list rest-result)))))
        (else (loop (cdr items)))))))

;; fold - reduce list to single value (left fold)
;; fold(f, init, lst) applies f(acc, x) for each x in lst
;; Works with Pyret tagged lists
(define (fold f init lst)
  (let ((unwrapped (pyret:unwrap-list lst)))
    (let loop ((acc init) (items unwrapped))
      (if (null? items)
          acc
          (loop (f acc (car items)) (cdr items))))))

;; each - apply function to each element for side effects, return nothing
;; Works with Pyret tagged lists
(define (each f lst)
  (let ((unwrapped (pyret:unwrap-list lst)))
    (let loop ((items unwrapped))
      (if (null? items)
          #f
          (begin
            (f (car items))
            (loop (cdr items)))))))

;; map2 - map over two lists simultaneously (cartesian product)
;; Works with Pyret tagged lists
(define (map2 f lst1 lst2)
  (let ((unwrapped1 (pyret:unwrap-list lst1))
        (unwrapped2 (pyret:unwrap-list lst2)))
    (let outer ((items1 unwrapped1))
      (if (null? items1)
          (pyret:wrap-list '())
          (let ((inner-results
                 (let inner ((items2 unwrapped2))
                   (if (null? items2)
                       '()
                       (cons (f (car items1) (car items2))
                             (inner (cdr items2)))))))
            (let ((rest-result (outer (cdr items1))))
              (pyret:wrap-list (append inner-results (pyret:unwrap-list rest-result)))))))))

;; ============================================================================
;; Object Support (Pyret objects/records)
;; ============================================================================

;; Pyret objects are represented as tagged association lists
;; Format: (pyret-object (field-name . value) (field-name . value) ...)
;; We use a tag to distinguish objects from plain lists

(define (pyret:make-object-literal . field-value-pairs)
  "Create a Pyret object from alternating field names and values"
  (let loop ((pairs field-value-pairs) (alist '()))
    (if (null? pairs)
        (cons 'pyret-object alist)
        (let ((field-name (car pairs))
              (field-value (cadr pairs)))
          (loop (cddr pairs)
                (append alist (list (cons field-name field-value))))))))

(define (pyret:object? x)
  "Check if x is a Pyret object"
  (and (pair? x) (eq? (car x) 'pyret-object)))

(define (pyret:object-get obj field-name)
  "Get a field value from a Pyret object by name"
  (if (not (pyret:object? obj))
      (error "Cannot access field on non-object" obj)
      (let ((alist (cdr obj)))
        (let ((pair (assoc field-name alist)))
          (if pair
              (cdr pair)
              (error "Object does not have field:" field-name))))))

(define (pyret:object-has-field? obj field-name)
  "Check if object has a field with given name"
  (if (not (pyret:object? obj))
      #f
      (let ((alist (cdr obj)))
        (if (assoc field-name alist) #t #f))))

(define (pyret:object-keys obj)
  "Get list of field names from object"
  (if (not (pyret:object? obj))
      (error "Cannot get keys from non-object" obj)
      (scheme:map car (cdr obj))))

;; ============================================================================
;; Option Type (some/none)
;; ============================================================================

;; Option type: some(value) or none
;; Represented as tagged values

(define pyret:none '(pyret-option-none))

(define (pyret:some value)
  (list 'pyret-option-some value))

(define (pyret:is-some opt)
  (and (pair? opt) (eq? (car opt) 'pyret-option-some)))

(define (pyret:is-none opt)
  (and (pair? opt) (eq? (car opt) 'pyret-option-none)))

(define (pyret:option-value opt)
  (if (pyret:is-some opt)
      (cadr opt)
      (error "Cannot get value from none")))

;; ============================================================================
;; Built-in Module: lists
;; ============================================================================

;; Map function over a list
(define (lists_builtin__map f lst)
  (if (pyret:empty? lst)
      lst
      (pyret:link (f (pyret:list-first lst))
                  (lists_builtin__map f (pyret:list-rest lst)))))

;; Filter list by predicate
(define (lists_builtin__filter f lst)
  (cond
    ((pyret:empty? lst) lst)
    ((f (pyret:list-first lst))
     (pyret:link (pyret:list-first lst)
                 (lists_builtin__filter f (pyret:list-rest lst))))
    (else (lists_builtin__filter f (pyret:list-rest lst)))))

;; Fold from the right
(define (lists_builtin__fold f base lst)
  (if (pyret:empty? lst)
      base
      (f (pyret:list-first lst)
         (lists_builtin__fold f base (pyret:list-rest lst)))))

;; Length of a list
(define (lists_builtin__length lst)
  (if (pyret:empty? lst)
      0
      (+ 1 (lists_builtin__length (pyret:list-rest lst)))))

;; Reverse a list
(define (lists_builtin__reverse lst)
  (define (rev-helper lst acc)
    (if (pyret:empty? lst)
        acc
        (rev-helper (pyret:list-rest lst)
                    (pyret:link (pyret:list-first lst) acc))))
  (rev-helper lst pyret:empty))

;; Get nth element (0-indexed)
(define (lists_builtin__get lst n)
  (if (pyret:empty? lst)
      (error "get: index out of bounds")
      (if (= n 0)
          (pyret:list-first lst)
          (lists_builtin__get (pyret:list-rest lst) (- n 1)))))

;; Append two lists
(define (lists_builtin__append front back)
  (if (pyret:empty? front)
      back
      (pyret:link (pyret:list-first front)
                  (lists_builtin__append (pyret:list-rest front) back))))

;; Range from start to stop (exclusive)
(define (lists_builtin__range start stop)
  (if (>= start stop)
      pyret:empty
      (pyret:link start (lists_builtin__range (+ start 1) stop))))

;; All elements satisfy predicate
(define (lists_builtin__all f lst)
  (cond
    ((pyret:empty? lst) #t)
    ((f (pyret:list-first lst))
     (lists_builtin__all f (pyret:list-rest lst)))
    (else #f)))

;; Any element satisfies predicate
(define (lists_builtin__any f lst)
  (cond
    ((pyret:empty? lst) #f)
    ((f (pyret:list-first lst)) #t)
    (else (lists_builtin__any f (pyret:list-rest lst)))))

;; ============================================================================
;; Built-in Module: option
;; ============================================================================

;; Option data type already defined in pyret:some and pyret:none above
;; Just need to add module-namespaced versions

(define (option_builtin__some x) (pyret:some x))
(define option_builtin__none pyret:none)
(define (option_builtin__is-some x) (pyret:is-some x))
(define (option_builtin__is-none x) (pyret:is-none x))

;; ============================================================================
;; Initialization
;; ============================================================================

;; Print a message when runtime is loaded (for debugging)
; (display "; Pyret runtime library loaded")
; (newline)
