% Represent natural numbers as zero followed by nested successor terms: zero
% is 0, s(zero) is 1, and s(s(zero)) is 2.
%
% Recursive rules define addition, multiplication, and factorial over this
% representation. With a complete sum supplied, addition can also enumerate
% the possible pairs of addends. The queries demonstrate both directions.

:- table add/3, multiply/3, factorial/2.

add(A, zero, A).
add(A, s(B), s(C)) :-
    add(A, B, C).

multiply(_, zero, zero).
multiply(A, s(B), C) :-
    multiply(A, B, D),
    add(A, D, C).

factorial(zero, s(zero)).
factorial(s(N), F) :-
    factorial(N, Before),
    multiply(s(N), Before, F).

% Answer and Reason
% Addition run backward enumerates every way of splitting a known sum.
?- add(A, B, s(s(s(zero)))).
?- why.
% One derivation chains all three relations: (1*2)+3 = 5, then 5! = 120.
?- multiply(s(zero), s(s(zero)), Product),
   add(Product, s(s(s(zero))), Sum),
   factorial(Sum, Factorial).
?- why.

% Check
:- assertion(aggregate_all(count, add(A, B, s(s(s(zero)))), 4)).
:- assertion(add(s(zero), s(s(zero)), s(s(s(zero))))).
:- assertion(multiply(s(zero), s(s(zero)), s(s(zero)))).
:- assertion(factorial(zero, s(zero))).
:- assertion(factorial(s(s(s(zero))), s(s(s(s(s(s(zero)))))))).
