% Differentiate expressions with respect to x using rules for constants,
% variables, sums, products, powers, and logarithms. Expressions are
% structured terms such as add(U, V) and mul(U, V), rather than text to be
% parsed.
%
% Results remain unsimplified: for example, differentiating x*x keeps the two
% product-rule terms 1*x and x*1 visible. The example constructs derivative
% expressions; it does not evaluate them at a numeric input.

:- table d/3.

expr(square, mul(var(x), var(x))).
expr(linear_plus_const, add(var(x), const(3))).
expr(product, mul(add(var(x), const(1)), mul(add(pow(var(x), 2), const(2)), add(pow(var(x), 3), const(3))))).
expr(nested_log, log(log(var(x)))).

d(const(_c), _x, const(0)).
d(var(X), X, const(1)).
d(var(Y), X, const(0)) :-
  (X \= Y).
d(add(U, V), X, add(Du, Dv)) :-
  d(U, X, Du),
  d(V, X, Dv).
d(sub(U, V), X, sub(Du, Dv)) :-
  d(U, X, Du),
  d(V, X, Dv).
d(mul(U, V), X, add(mul(Du, V), mul(U, Dv))) :-
  d(U, X, Du),
  d(V, X, Dv).
d(divide(U, V), X, divide(sub(mul(Du, V), mul(U, Dv)), pow(V, 2))) :-
  d(U, X, Du),
  d(V, X, Dv).
d(pow(U, N), X, mul(mul(const(N), pow(U, N1)), Du)) :-
  (N1 is N - 1),
  d(U, X, Du).
d(log(U), X, divide(Du, U)) :-
  d(U, X, Du).

derivative_result(Name, Derivative) :-
  expr(Name, Expr),
  d(Expr, x, Derivative).

% Answer and Reason
?- derivative_result(Name, Derivative).
?- why(derivative_result(square, Derivative)).

% Check
:- assertion(derivative_result(square, add(mul(const(1), var(x)), mul(var(x), const(1))))).
:- assertion(derivative_result(linear_plus_const, add(const(1), const(0)))).
:- assertion(derivative_result(nested_log, divide(divide(const(1), var(x)), log(var(x))))).
:- assertion(aggregate_all(count, derivative_result(Name, Derivative), 4)).
:- assertion(d(var(y), x, const(0))).
:- assertion(d(pow(var(x), 3), x, mul(mul(const(3), pow(var(x), 2)), const(1)))).
