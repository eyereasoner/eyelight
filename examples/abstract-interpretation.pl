% Find possible division by zero by tracking whether a value is negative,
% zero, or positive, rather than tracking every possible integer.
%
% The countdown program repeatedly subtracts one before dividing; the guarded
% program divides only when its value is positive. The analysis combines the
% possible signs at each program point until no new signs appear. A warning
% means zero is possible in this analysis. Only the listed operations and
% conditions are modeled, using integers without machine overflow.

program(countdown).
program(guarded).
entry(countdown, input).
entry(guarded, input).
input_sign(neg).
input_sign(zero).
input_sign(pos).
transition(countdown, input, negative_branch, negative, negate).
transition(countdown, input, nonnegative_branch, nonnegative, identity).
transition(countdown, negative_branch, loop_head, any, identity).
transition(countdown, nonnegative_branch, loop_head, any, identity).
transition(countdown, loop_head, loop_body, positive, decrement).
transition(countdown, loop_body, loop_head, any, identity).
transition(countdown, loop_head, divide, nonpositive, identity).
transition(guarded, input, negative_branch, negative, negate).
transition(guarded, input, nonnegative_branch, nonnegative, identity).
transition(guarded, negative_branch, check, any, identity).
transition(guarded, nonnegative_branch, check, any, identity).
transition(guarded, check, divide, positive, identity).
transition(guarded, check, zero_result, nonpositive, identity).
division(countdown, divide).
division(guarded, divide).

accepts(any, neg).
accepts(any, zero).
accepts(any, pos).
accepts(negative, neg).
accepts(nonnegative, zero).
accepts(nonnegative, pos).
accepts(positive, pos).
accepts(nonpositive, neg).
accepts(nonpositive, zero).
transfer(identity, neg, neg).
transfer(identity, zero, zero).
transfer(identity, pos, pos).
transfer(negate, neg, pos).
transfer(negate, zero, zero).
transfer(negate, pos, neg).
transfer(decrement, neg, neg).
transfer(decrement, zero, neg).
transfer(decrement, pos, zero).
transfer(decrement, pos, pos).

abs_state(Program, Point, Sign) :- entry(Program, Point), input_sign(Sign).
abs_state(Program, To, Result) :-
  abs_state(Program, From, Sign), transition(Program, From, To, Guard, Operation),
  accepts(Guard, Sign), transfer(Operation, Sign, Result).
zero_possible(Program, Point) :- abs_state(Program, Point, zero).
reachable_division(Program, Point) :- division(Program, Point), abs_state(Program, Point, _).
abs_warning(Program, possible_division_by_zero(Point)) :- division(Program, Point), zero_possible(Program, Point).
nonzero_divisor(Program, Point) :- reachable_division(Program, Point), \+ zero_possible(Program, Point).

% Answer and Reason
?- abs_state(Program, Point, Sign).
?- abs_warning(Program, Warning).
?- why.
?- nonzero_divisor(Program, Point).
?- why.

% Check
:- assertion(abs_warning(countdown, possible_division_by_zero(divide))).
:- assertion(abs_state(countdown, loop_head, zero)).
:- assertion(abs_state(countdown, loop_head, pos)).
:- assertion(\+ abs_state(countdown, loop_head, neg)).
:- assertion(abs_state(countdown, divide, zero)).
:- assertion(abs_state(guarded, divide, pos)).
:- assertion(nonzero_divisor(guarded, divide)).
:- assertion(\+ nonzero_divisor(countdown, divide)).
:- assertion(\+ abs_warning(guarded, possible_division_by_zero(divide))).
:- assertion(aggregate_all(count, abs_warning(Program, Warning), 1)).
