(define (not x)            (if x #f #t))
(define (null? obj)        (if (eqv? obj '()) #t #f))

(define (factorial x) (if (= x 1) 1 (* x (factorial (- x 1)))))

