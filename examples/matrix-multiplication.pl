% Multiply two 2-by-2 matrices using exact integer arithmetic. The term m(A,
% B, C, D) stores the rows [A, B] and [C, D].
%
% The sample calculates A*B and B*A and shows that the results differ: matrix
% multiplication can depend on order. Their difference, A*B - B*A, is called
% the commutator.

matrix(a, m(1, 2, 0, 1)).
matrix(b, m(1, 0, 3, 1)).

:- table multiply/3, commutator/3.
multiply(m(A, B, C, D), m(E, F, G, H), m(I, J, K, L)) :-
  I is A*E + B*G,
  J is A*F + B*H,
  K is C*E + D*G,
  L is C*F + D*H.

commutator(A, B, m(I, J, K, L)) :-
  multiply(A, B, m(P, Q, R, S)),
  multiply(B, A, m(T, U, V, W)),
  I is P-T, J is Q-U, K is R-V, L is S-W.

product(Left, Right, Product) :-
  matrix(Left, A), matrix(Right, B), multiply(A, B, Product).

% Answer and Reason
?- product(a, b, Product).
?- why.
?- product(b, a, Product).
?- matrix(a, A), matrix(b, B), commutator(A, B, Difference).

% Check
:- assertion(product(a, b, m(7, 2, 3, 1))).
:- assertion(product(b, a, m(1, 2, 3, 7))).
:- assertion((product(a, b, AB), product(b, a, BA), AB \= BA)).
:- assertion((matrix(a, A), multiply(A, m(1, 0, 0, 1), A))).
:- assertion((matrix(a, A), matrix(b, B), commutator(A, B, m(6, 0, 0, -6)))).
