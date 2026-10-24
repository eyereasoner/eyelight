% Summarize three players' scores using a count, sum, minimum, and maximum.
% Repeating Luna's score fact does not count it twice. Milo and Luna still
% contribute separately, even though their scores are equal, because they are
% different players.

is_a(milo, player).
is_a(luna, player).
is_a(rex, player).
score(milo, 7).
score(luna, 7).
score(rex, 10).
score(luna, 7).

is_a(Player, 'team-member') :-
    is_a(Player, player).

% Answer and Reason
?- aggregate_all(count, is_a(Subject, player), Result).
?- aggregate_all(count, is_a(Subject, 'team-member'), Result).
?- aggregate_all(sum(Value), score(Subject, Value), Result).
?- why.
?- aggregate_all(min(Value), score(Subject, Value), Result).
?- aggregate_all(max(Value), score(Subject, Value), Result).

% Check
:- assertion((aggregate_all(count, is_a(Subject, player), 3))).
:- assertion((aggregate_all(count, is_a(Subject, 'team-member'), 3))).
:- assertion((aggregate_all(sum(Value), score(Subject, Value), 24))).
:- assertion((aggregate_all(min(Value), score(Subject, Value), 7))).
:- assertion((aggregate_all(max(Value), score(Subject, Value), 10))).
