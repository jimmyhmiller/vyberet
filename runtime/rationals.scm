;;; Rational Number Support for Ribbit Scheme
;;; This file is only loaded when compiling to Ribbit backend
;;; Ribbit only supports integer arithmetic, so we implement rationals as tagged pairs

;; ============================================================================
;; Rational Data Structure
;; ============================================================================

;; Rationals are represented as: (cons 'RATIONAL (cons numerator denominator))
;; This allows us to distinguish them from regular pairs

;; Override the pyret:rat? definition from runtime.scm
;; For Ribbit, we check for our tagged pair representation
(define (pyret:rat? x)
  (and (pair? x)
       (pair? (cdr x))
       (eq? (car x) 'RATIONAL)))

;; Accessors
(define (rat-num r)
  (if (pyret:rat? r)
      (cadr r)
      (error "rat-num: not a rational")))

(define (rat-den r)
  (if (pyret:rat? r)
      (cddr r)
      (error "rat-den: not a rational")))

;; Constructor with automatic simplification
(define (make-rat n d)
  (if (= d 0)
      (error "Division by zero")
      (let* ((g (gcd (abs n) (abs d)))
             (n0 (quotient n g))
             (d0 (quotient d g)))
        ;; Ensure denominator is always positive
        (if (< d0 0)
            (cons 'RATIONAL (cons (- 0 n0) (- 0 d0)))
            (cons 'RATIONAL (cons n0 d0))))))

;; Convert integer to rational
(define (int->rat n)
  (make-rat n 1))

;; Normalize: if input is already a rational, return it; otherwise make it one
(define (->rat x)
  (if (pyret:rat? x)
      x
      (int->rat x)))

;; ============================================================================
;; Arithmetic Operations
;; ============================================================================

(define (rat+ x y)
  (let ((x-rat (->rat x))
        (y-rat (->rat y)))
    (make-rat (+ (* (rat-num x-rat) (rat-den y-rat))
                 (* (rat-num y-rat) (rat-den x-rat)))
              (* (rat-den x-rat) (rat-den y-rat)))))

(define (rat- x y)
  (let ((x-rat (->rat x))
        (y-rat (->rat y)))
    (make-rat (- (* (rat-num x-rat) (rat-den y-rat))
                 (* (rat-num y-rat) (rat-den x-rat)))
              (* (rat-den x-rat) (rat-den y-rat)))))

(define (rat* x y)
  (let ((x-rat (->rat x))
        (y-rat (->rat y)))
    (make-rat (* (rat-num x-rat) (rat-num y-rat))
              (* (rat-den x-rat) (rat-den y-rat)))))

(define (rat/ x y)
  (let ((x-rat (->rat x))
        (y-rat (->rat y)))
    (if (= (rat-num y-rat) 0)
        (error "Division by zero")
        (make-rat (* (rat-num x-rat) (rat-den y-rat))
                  (* (rat-den x-rat) (rat-num y-rat))))))

(define (rat-negate x)
  (let ((x-rat (->rat x)))
    (make-rat (- 0 (rat-num x-rat)) (rat-den x-rat))))

;; ============================================================================
;; Comparison Operations
;; ============================================================================

(define (rat= x y)
  (let ((x-rat (->rat x))
        (y-rat (->rat y)))
    (and (= (rat-num x-rat) (rat-num y-rat))
         (= (rat-den x-rat) (rat-den y-rat)))))

(define (rat< x y)
  (let ((x-rat (->rat x))
        (y-rat (->rat y)))
    ;; a/b < c/d  iff  a*d < c*b  (assuming positive denominators)
    (< (* (rat-num x-rat) (rat-den y-rat))
       (* (rat-num y-rat) (rat-den x-rat)))))

(define (rat> x y)
  (rat< y x))

(define (rat<= x y)
  (not (rat> x y)))

(define (rat>= x y)
  (not (rat< x y)))

;; ============================================================================
;; Display Support
;; ============================================================================

(define (display-rat r)
  (if (pyret:rat? r)
      (if (= (rat-den r) 1)
          ; If denominator is 1, just display the numerator (it's an integer)
          (display (rat-num r))
          ; Otherwise display as fraction
          (begin
            (display (rat-num r))
            (display "/")
            (display (rat-den r))))
      (error "display-rat: not a rational")))

;; ============================================================================
;; Conversion
;; ============================================================================

;; Convert rational to approximate floating point (not available in Ribbit)
;; This is here for potential future use, but Ribbit doesn't support floats
(define (rat->inexact r)
  (let ((r-rat (->rat r)))
    (quotient (rat-num r-rat) (rat-den r-rat))))  ; Best we can do: integer division

;; Simplify rational (already done in make-rat, but exposed for explicit use)
(define (rat-simplify r)
  (if (pyret:rat? r)
      (make-rat (rat-num r) (rat-den r))
      (->rat r)))

;; ============================================================================
;; Polymorphic Operations (for compatibility with pyret:+ etc.)
;; ============================================================================

;; Polymorphic addition: handles both rationals and string concatenation
(define (pyret:+ a b)
  (cond
    ((and (string? a) (string? b)) (string-append a b))
    ((or (string? a) (string? b))
     (error "Cannot add string and number"))
    (else (rat+ a b))))

;; Polymorphic equality: handles rationals, integers, strings, booleans, lists, etc.
(define (pyret:equal? x y)
  (cond
    ;; Both rationals
    ((and (pyret:rat? x) (pyret:rat? y)) (rat= x y))
    ;; One rational, one integer - convert integer to rational
    ((and (pyret:rat? x) (number? y)) (rat= x (make-rat y 1)))
    ((and (number? x) (pyret:rat? y)) (rat= (make-rat x 1) y))
    ;; Lists - compare element by element
    ((and (pair? x) (pair? y))
     (and (pyret:equal? (car x) (car y))
          (pyret:equal? (cdr x) (cdr y))))
    ((and (null? x) (null? y)) #t)
    ;; Other types - use standard equal?
    (else (equal? x y))))

;; Polymorphic inequality
(define (pyret:not-equal? x y)
  (not (pyret:equal? x y)))
